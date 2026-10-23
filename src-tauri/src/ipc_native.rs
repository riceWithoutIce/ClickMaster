//! Read-only target lease plus native transport. Only the authorized worker calls transport.
use crate::{ipc, targets::Identity};
use ipc::{Failure, FailureReason as Reason, FailureStage as Stage};
use std::{ffi::OsString, fs, mem, os::windows::ffi::OsStringExt, path::PathBuf, ptr};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, GetLastError, LocalFree, ERROR_IO_PENDING, ERROR_PIPE_CONNECTED, FILETIME,
        HANDLE, HWND, INVALID_HANDLE_VALUE, LPARAM, RECT,
    },
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        },
        Cryptography::*,
        GetTokenInformation, TokenUser, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    },
    Storage::FileSystem::{
        FlushFileBuffers, WriteFile, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED,
        PIPE_ACCESS_DUPLEX,
    },
    System::{
        Diagnostics::ToolHelp::*,
        Pipes::*,
        Threading::*,
        IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED},
    },
    UI::WindowsAndMessaging::*,
};

pub struct Handle(pub HANDLE);
unsafe impl Send for Handle {}
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
impl Handle {
    pub fn new(h: HANDLE) -> Result<Self, String> {
        if h.is_null() || h == INVALID_HANDLE_VALUE {
            Err("无法打开必要的系统资源。".into())
        } else {
            Ok(Self(h))
        }
    }
}
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub fn random() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    if unsafe {
        BCryptGenRandom(
            ptr::null_mut(),
            bytes.as_mut_ptr(),
            32,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    } < 0
    {
        return Err("无法生成 worker 授权。".into());
    }
    Ok(hex(&bytes))
}
pub fn random_u32() -> Result<u32, String> {
    let mut bytes = [0; 4];
    if unsafe {
        BCryptGenRandom(
            ptr::null_mut(),
            bytes.as_mut_ptr(),
            4,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    } < 0
    {
        Err("随机源不可用，执行已停止。".into())
    } else {
        Ok(u32::from_ne_bytes(bytes))
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}
pub fn sha(bytes: &[u8]) -> Result<String, String> {
    let mut algorithm = ptr::null_mut();
    unsafe {
        if BCryptOpenAlgorithmProvider(&mut algorithm, wide("SHA256").as_ptr(), ptr::null(), 0) < 0
        {
            return Err("无法校验版本。".into());
        }
        let mut digest = [0u8; 32];
        let status = BCryptHash(
            algorithm,
            ptr::null(),
            0,
            bytes.as_ptr(),
            bytes.len().try_into().map_err(|_| "文件过大。")?,
            digest.as_mut_ptr(),
            32,
        );
        BCryptCloseAlgorithmProvider(algorithm, 0);
        if status < 0 {
            Err("无法校验版本。".into())
        } else {
            Ok(hex(&digest))
        }
    }
}
pub fn file_sha(path: &std::path::Path) -> Result<String, String> {
    sha(&fs::read(path).map_err(|_| "无法读取版本文件，请检查权限与目标安装。")?)
}
pub fn process(pid: u32) -> Result<(Handle, u64, PathBuf), Failure> {
    unsafe {
        let h = Handle::new(OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            pid,
        ))
        .map_err(|_| {
            Failure::new(Stage::Target, Reason::ProcessUnavailable).code(GetLastError())
        })?;
        let mut times: [FILETIME; 4] = mem::zeroed();
        if GetProcessTimes(
            h.0,
            &mut times[0],
            &mut times[1],
            &mut times[2],
            &mut times[3],
        ) == 0
        {
            return Err(
                Failure::new(Stage::Target, Reason::ProcessUnavailable).code(GetLastError())
            );
        }
        let mut buf = vec![0u16; 32768];
        let mut len = buf.len() as u32;
        if QueryFullProcessImageNameW(h.0, 0, buf.as_mut_ptr(), &mut len) == 0 {
            return Err(
                Failure::new(Stage::Target, Reason::ProcessUnavailable).code(GetLastError())
            );
        }
        let created = (times[0].dwHighDateTime as u64) << 32 | times[0].dwLowDateTime as u64;
        Ok((
            h,
            created,
            PathBuf::from(OsString::from_wide(&buf[..len as usize])),
        ))
    }
}
pub fn running(h: &Handle) -> bool {
    unsafe { WaitForSingleObject(h.0, 0) == 258 }
}
fn unique_process(pid: u32) -> Result<(), Failure> {
    unsafe {
        let snapshot =
            Handle::new(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)).map_err(|_| {
                Failure::new(Stage::Target, Reason::ProcessEnumeration).code(GetLastError())
            })?;
        let mut item: PROCESSENTRY32W = mem::zeroed();
        item.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snapshot.0, &mut item) == 0 {
            return Err(
                Failure::new(Stage::Target, Reason::ProcessEnumeration).code(GetLastError())
            );
        }
        let mut matching = Vec::new();
        loop {
            let n = item
                .szExeFile
                .iter()
                .position(|v| *v == 0)
                .unwrap_or(item.szExeFile.len());
            if String::from_utf16_lossy(&item.szExeFile[..n]).eq_ignore_ascii_case("TapTapLoot.exe")
            {
                matching.push(item.th32ProcessID);
            }
            if Process32NextW(snapshot.0, &mut item) == 0 {
                let code = GetLastError();
                if code != windows_sys::Win32::Foundation::ERROR_NO_MORE_FILES {
                    return Err(Failure::new(Stage::Target, Reason::ProcessEnumeration).code(code));
                }
                break;
            }
        }
        if matching == [pid] {
            Ok(())
        } else {
            Err(Failure::new(Stage::Target, Reason::MultipleInstances))
        }
    }
}
struct Windows {
    pid: u32,
    windows: Vec<usize>,
}
unsafe extern "system" fn collect(hwnd: HWND, param: LPARAM) -> i32 {
    let ctx = &mut *(param as *mut Windows);
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    let mut class = [0u16; 128];
    let n = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32);
    if pid == ctx.pid && String::from_utf16_lossy(&class[..n.max(0) as usize]) == "UnityWndClass" {
        ctx.windows.push(hwnd as usize);
    }
    1
}
pub struct Lease {
    pub identity: Identity,
    process: Handle,
}
fn supported_identity(created: u64, expected: u64, basename: &str) -> bool {
    created == expected && basename.eq_ignore_ascii_case("TapTapLoot.exe")
}
fn direct_window_policy(
    identity_matches: bool,
    visible: bool,
    minimized: bool,
    width: i32,
    height: i32,
) -> Result<(), Failure> {
    let reason = if !identity_matches {
        Some(Reason::WindowIdentity)
    } else if !visible {
        Some(Reason::WindowHidden)
    } else if minimized {
        Some(Reason::WindowMinimized)
    } else if width <= 0 || height <= 0 {
        Some(Reason::EmptyClient)
    } else {
        None
    };
    match reason {
        Some(reason) => Err(Failure::new(Stage::Target, reason)),
        None => Ok(()),
    }
}
fn full_guard(
    fast: impl FnOnce() -> Result<(), Failure>,
    unique: impl FnOnce() -> Result<(), Failure>,
    windows: impl FnOnce() -> Result<(), Failure>,
) -> Result<(), Failure> {
    fast()?;
    unique()?;
    windows()
}
impl Lease {
    pub fn open(identity: &Identity) -> Result<Self, Failure> {
        let (process, created, exe) = process(identity.pid)?;
        if !supported_identity(
            created,
            identity.created,
            &exe.file_name().unwrap_or_default().to_string_lossy(),
        ) {
            return Err(Failure::new(Stage::Target, Reason::InstanceChanged));
        }
        let dll = exe
            .parent()
            .ok_or_else(|| Failure::new(Stage::Compatibility, Reason::VersionUnreadable))?
            .join("TapTapLoot_Data/Managed/Assembly-CSharp.dll");
        if {
            let _measure = crate::performance::span(crate::performance::Stage::AssemblyHash);
            file_sha(&dll)
        }
        .map_err(|_| Failure::new(Stage::Compatibility, Reason::VersionUnreadable))?
            != ipc::ASSEMBLY_SHA256
        {
            return Err(Failure::new(
                Stage::Compatibility,
                Reason::UnsupportedVersion,
            ));
        }
        let lease = Self {
            identity: identity.clone(),
            process,
        };
        lease.validate_fast()?;
        Ok(lease)
    }
    /// Hot-loop check uses the retained process and direct window only. Focus is irrelevant.
    pub fn validate_fast(&self) -> Result<(), Failure> {
        unsafe {
            match WaitForSingleObject(self.process.0, 0) {
                258 => {}
                0 => return Err(Failure::new(Stage::Target, Reason::ProcessExited)),
                _ => {
                    return Err(Failure::new(Stage::Target, Reason::ProcessUnavailable)
                        .code(GetLastError()))
                }
            }
            let hwnd = self.identity.hwnd as HWND;
            if IsWindow(hwnd) == 0 {
                return Err(Failure::new(Stage::Target, Reason::WindowInvalid));
            }
            let mut pid = 0;
            let thread = GetWindowThreadProcessId(hwnd, &mut pid);
            if thread == 0 {
                return Err(Failure::new(Stage::Target, Reason::WindowInvalid).code(GetLastError()));
            }
            let identity_matches = pid == self.identity.pid && thread == self.identity.thread;
            let visible = IsWindowVisible(hwnd) != 0;
            let minimized = IsIconic(hwnd) != 0;
            let mut rect: RECT = mem::zeroed();
            if GetClientRect(hwnd, &mut rect) == 0 {
                return Err(Failure::new(Stage::Target, Reason::EmptyClient).code(GetLastError()));
            }
            direct_window_policy(
                identity_matches,
                visible,
                minimized,
                rect.right,
                rect.bottom,
            )?;
        }
        Ok(())
    }
    /// Called once at admission and once at the worker's final send boundary.
    pub fn validate_for_send(&self) -> Result<(), Failure> {
        let _measure = crate::performance::span(crate::performance::Stage::FullValidation);
        full_guard(
            || self.validate_fast(),
            || unique_process(self.identity.pid),
            || self.unique_window(),
        )
    }
    fn unique_window(&self) -> Result<(), Failure> {
        let mut ctx = Windows {
            pid: self.identity.pid,
            windows: Vec::new(),
        };
        unsafe {
            if EnumWindows(Some(collect), &mut ctx as *mut _ as isize) == 0 {
                return Err(
                    Failure::new(Stage::Target, Reason::WindowEnumeration).code(GetLastError())
                );
            }
            if ctx.windows != [self.identity.hwnd] {
                return Err(Failure::new(Stage::Target, Reason::WindowAmbiguous));
            }
        }
        Ok(())
    }
}

struct Local(*mut core::ffi::c_void);
impl Drop for Local {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
fn user_sddl() -> Result<Vec<u16>, Failure> {
    unsafe {
        let mut raw = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) == 0 {
            return Err(Failure::new(Stage::PipeCreate, Reason::SystemCall).code(GetLastError()));
        }
        let token = Handle::new(raw).map_err(|_| {
            Failure::new(Stage::PipeCreate, Reason::SystemCall).code(GetLastError())
        })?;
        let mut len = 0;
        GetTokenInformation(token.0, TokenUser, ptr::null_mut(), 0, &mut len);
        let mut buffer = vec![0usize; (len as usize).div_ceil(mem::size_of::<usize>())];
        if GetTokenInformation(
            token.0,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            len,
            &mut len,
        ) == 0
        {
            return Err(Failure::new(Stage::PipeCreate, Reason::SystemCall).code(GetLastError()));
        }
        let sid = (*(buffer.as_ptr() as *const TOKEN_USER)).User.Sid;
        let mut text = ptr::null_mut();
        if ConvertSidToStringSidW(sid, &mut text) == 0 {
            return Err(Failure::new(Stage::PipeCreate, Reason::SystemCall).code(GetLastError()));
        }
        let free = Local(text.cast());
        let mut n = 0;
        while *text.add(n) != 0 {
            n += 1;
        }
        let sddl = wide(&format!(
            "D:P(A;;GA;;;{})",
            String::from_utf16_lossy(std::slice::from_raw_parts(text, n))
        ));
        drop(free);
        Ok(sddl)
    }
}
pub fn pipe(name: &str) -> Result<Handle, Failure> {
    let sddl = user_sddl()?;
    unsafe {
        let mut descriptor = ptr::null_mut();
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            ptr::null_mut(),
        ) == 0
        {
            return Err(Failure::new(Stage::PipeCreate, Reason::SystemCall).code(GetLastError()));
        }
        let _free = Local(descriptor);
        let attributes = SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        Handle::new(CreateNamedPipeW(
            wide(name).as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_REJECT_REMOTE_CLIENTS,
            1,
            4096,
            4096,
            0,
            &attributes,
        ))
        .map_err(|_| Failure::new(Stage::PipeCreate, Reason::SystemCall).code(GetLastError()))
    }
}
pub fn connect(pipe: &Handle) -> Result<(), Failure> {
    let _measure = crate::performance::span(crate::performance::Stage::Connect);
    connect_bounded(pipe, ipc::CONNECT_MS as u32)
}
fn finish(pipe: &Handle, ov: &OVERLAPPED, timeout: u32, stage: Stage) -> Result<u32, Failure> {
    unsafe {
        let mut bytes = 0;
        let wait = WaitForSingleObject(ov.hEvent, timeout);
        if wait != 0 {
            let failure = if wait == 258 {
                Failure::new(stage, Reason::Timeout)
            } else {
                Failure::new(stage, Reason::SystemCall).code(GetLastError())
            };
            CancelIoEx(pipe.0, ov);
            // Keep OVERLAPPED/event storage alive until the cancelled operation is retired.
            GetOverlappedResult(pipe.0, ov, &mut bytes, 1);
            return Err(failure);
        }
        if GetOverlappedResult(pipe.0, ov, &mut bytes, 0) == 0 {
            Err(Failure::new(stage, Reason::SystemCall).code(GetLastError()))
        } else {
            Ok(bytes)
        }
    }
}
pub fn connect_bounded(pipe: &Handle, timeout: u32) -> Result<(), Failure> {
    unsafe {
        let event = Handle::new(CreateEventW(ptr::null(), 1, 0, ptr::null()))
            .map_err(|_| Failure::new(Stage::Connect, Reason::SystemCall).code(GetLastError()))?;
        let mut ov: OVERLAPPED = mem::zeroed();
        ov.hEvent = event.0;
        if ConnectNamedPipe(pipe.0, &mut ov) != 0 {
            return Ok(());
        }
        match GetLastError() {
            ERROR_PIPE_CONNECTED => Ok(()),
            ERROR_IO_PENDING => finish(pipe, &ov, timeout, Stage::Connect).map(|_| ()),
            code => Err(Failure::new(Stage::Connect, Reason::SystemCall).code(code)),
        }
    }
}
pub fn client(pipe: &Handle, lease: &Lease) -> Result<(), Failure> {
    lease.validate_fast()?;
    let mut pid = 0;
    if unsafe { GetNamedPipeClientProcessId(pipe.0, &mut pid) } == 0 {
        return Err(Failure::new(Stage::Client, Reason::SystemCall).code(unsafe { GetLastError() }));
    }
    check_client(pid, lease.identity.pid)
}
fn check_client(actual: u32, expected: u32) -> Result<(), Failure> {
    if actual != expected {
        Err(Failure::new(Stage::Client, Reason::ClientMismatch))
    } else {
        Ok(())
    }
}
pub fn write(pipe: &Handle) -> Result<(), Failure> {
    let _measure = crate::performance::span(crate::performance::Stage::Write);
    unsafe {
        let event = Handle::new(CreateEventW(ptr::null(), 1, 0, ptr::null()))
            .map_err(|_| Failure::new(Stage::Write, Reason::SystemCall).code(GetLastError()))?;
        let mut ov: OVERLAPPED = mem::zeroed();
        ov.hEvent = event.0;
        let mut bytes = 0;
        if WriteFile(pipe.0, ipc::FRAME.as_ptr(), 4, &mut bytes, &mut ov) == 0 {
            let code = GetLastError();
            if code != ERROR_IO_PENDING {
                return Err(Failure::new(Stage::Write, Reason::SystemCall).code(code));
            }
            bytes = finish(pipe, &ov, ipc::WRITE_MS as u32, Stage::Write)?;
        }
        if bytes != 4 {
            return Err(Failure::new(Stage::Write, Reason::WriteIncomplete));
        }
        Ok(())
    }
}
pub fn drain(pipe: &Handle) -> Result<(), u32> {
    let _measure = crate::performance::span(crate::performance::Stage::ReadObservation);
    if unsafe { FlushFileBuffers(pipe.0) } == 0 {
        Err(unsafe { GetLastError() })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_gate_requires_exact_executable_and_original_creation_time() {
        assert!(supported_identity(123, 123, "TapTapLoot.exe"));
        assert!(!supported_identity(124, 123, "TapTapLoot.exe"));
        assert!(!supported_identity(123, 123, "TapTapLoot-helper.exe"));
        assert!(!supported_identity(123, 123, "Other.exe"));
    }
    #[test]
    fn direct_window_policy_preserves_identity_and_minimization_without_a_focus_gate() {
        assert!(direct_window_policy(true, true, false, 16, 16).is_ok());
        assert_eq!(
            direct_window_policy(false, true, false, 16, 16)
                .unwrap_err()
                .reason,
            Reason::WindowIdentity
        );
        assert_eq!(
            direct_window_policy(true, true, true, 16, 16)
                .unwrap_err()
                .reason,
            Reason::WindowMinimized
        );
        assert_eq!(
            direct_window_policy(true, false, false, 16, 16)
                .unwrap_err()
                .reason,
            Reason::WindowHidden
        );
        assert_eq!(
            direct_window_policy(true, true, false, 0, 16)
                .unwrap_err()
                .reason,
            Reason::EmptyClient
        );
        assert!(check_client(5, 5).is_ok());
        assert_eq!(
            check_client(6, 5).unwrap_err().reason,
            Reason::ClientMismatch
        );
    }
    #[test]
    fn final_guard_enumerates_once_and_short_circuits_before_later_checks() {
        use std::cell::RefCell;
        let calls = RefCell::new(Vec::new());
        full_guard(
            || {
                calls.borrow_mut().push("fast");
                Ok(())
            },
            || {
                calls.borrow_mut().push("process");
                Ok(())
            },
            || {
                calls.borrow_mut().push("window");
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(*calls.borrow(), ["fast", "process", "window"]);
        let failure = Failure::new(Stage::Target, Reason::ProcessExited);
        assert_eq!(
            full_guard(
                || Err(failure.clone()),
                || panic!("no process scan after fast failure"),
                || panic!("no window scan after fast failure")
            )
            .unwrap_err(),
            failure
        );
    }
}
