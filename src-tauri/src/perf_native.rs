//! Resource queries on handles owned by this process. No process-name scans.
use crate::ipc_native::Handle;
use std::{mem, os::windows::io::AsRawHandle, process::Child, ptr};
use windows_sys::Win32::{
    Foundation::{DuplicateHandle, DUPLICATE_SAME_ACCESS, FILETIME, HANDLE},
    System::{
        ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX},
        Threading::*,
    },
};

pub fn cpu(handle: HANDLE) -> Option<u64> {
    unsafe {
        let mut t: [FILETIME; 4] = mem::zeroed();
        if GetProcessTimes(handle, &mut t[0], &mut t[1], &mut t[2], &mut t[3]) == 0 {
            return None;
        }
        let ticks = |t: FILETIME| ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64;
        Some(ticks(t[2]).saturating_add(ticks(t[3])))
    }
}
pub fn current_cpu() -> Option<u64> {
    cpu(unsafe { GetCurrentProcess() })
}
pub fn memory() -> Option<(u64, u64)> {
    process_memory(unsafe { GetCurrentProcess() })
}
pub fn process_memory(handle: HANDLE) -> Option<(u64, u64)> {
    unsafe {
        let mut m: PROCESS_MEMORY_COUNTERS_EX = mem::zeroed();
        m.cb = mem::size_of_val(&m) as u32;
        if GetProcessMemoryInfo(handle, &mut m as *mut _ as _, m.cb) == 0 {
            None
        } else {
            Some((m.WorkingSetSize as u64, m.PrivateUsage as u64))
        }
    }
}
/// PID is used only immediately after the official environment enumeration; never persisted.
/// The creation stamp and original owned handle prevent subsequent PID reuse attribution.
pub fn open_process(pid: u32) -> Option<(Handle, u64)> {
    let handle = Handle::new(unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_SYNCHRONIZE,
            0,
            pid,
        )
    })
    .ok()?;
    let stamp = creation(handle.0)?;
    if exited(&handle) {
        None
    } else {
        Some((handle, stamp))
    }
}
pub fn creation(handle: HANDLE) -> Option<u64> {
    unsafe {
        let mut t: [FILETIME; 4] = mem::zeroed();
        if GetProcessTimes(handle, &mut t[0], &mut t[1], &mut t[2], &mut t[3]) == 0 {
            None
        } else {
            Some(((t[0].dwHighDateTime as u64) << 32) | t[0].dwLowDateTime as u64)
        }
    }
}
pub fn duplicate(child: &Child) -> Option<Handle> {
    unsafe {
        let mut out = ptr::null_mut();
        let me = GetCurrentProcess();
        if DuplicateHandle(
            me,
            child.as_raw_handle(),
            me,
            &mut out,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        ) == 0
        {
            None
        } else {
            Handle::new(out).ok()
        }
    }
}
pub fn exited(handle: &Handle) -> bool {
    unsafe { WaitForSingleObject(handle.0, 0) == 0 }
}

/// Share mode zero gives cross-instance ownership; a crash releases the handle.
pub fn directory_lock(path: &std::path::Path) -> Option<Handle> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{
        Foundation::{GENERIC_READ, GENERIC_WRITE},
        Storage::FileSystem::*,
    };
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    Handle::new(unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            ptr::null(),
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        )
    })
    .ok()
}
