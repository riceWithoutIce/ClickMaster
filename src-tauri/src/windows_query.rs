//! Read-only Windows query adapter. No target activation or mutation.
use crate::targets::{
    Enumeration, Identity, ObservedWindow, QueryError, WindowQuery, WindowStatus,
};
use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::Path};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_ACCESS_DENIED, FILETIME, HANDLE, HWND, LPARAM},
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
    System::Threading::{
        GetExitCodeProcess, GetProcessTimes, OpenProcess, QueryFullProcessImageNameW,
        PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetDesktopWindow, GetShellWindow, GetWindow, GetWindowLongPtrW,
        GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow,
        IsWindowVisible, GWL_EXSTYLE, GW_OWNER, WS_EX_APPWINDOW, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW,
    },
};

pub struct NativeQuery;
struct Process(HANDLE);
impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn main_window() -> WindowProperties {
        WindowProperties {
            visible: true,
            title: true,
            ..Default::default()
        }
    }
    #[test]
    fn range_allows_hidden_and_untitled_but_keeps_helper_exclusions() {
        assert!(eligible(&main_window(), false));
        let owned_app = WindowProperties {
            owned: true,
            appwindow: true,
            ..main_window()
        };
        assert!(eligible(&owned_app, false));
        for window in [
            WindowProperties {
                visible: false,
                ..main_window()
            },
            WindowProperties {
                title: false,
                ..main_window()
            },
            WindowProperties {
                cloaked: true,
                ..main_window()
            },
            WindowProperties::default(),
        ] {
            assert!(!eligible(&window, false));
            assert!(eligible(&window, true));
        }
        for window in [
            WindowProperties {
                own: true,
                ..main_window()
            },
            WindowProperties {
                owned: true,
                ..main_window()
            },
            WindowProperties {
                noactivate: true,
                ..main_window()
            },
            WindowProperties {
                system_shell: true,
                ..main_window()
            },
        ] {
            assert!(!eligible(&window, false));
            assert!(!eligible(&window, true));
        }
    }
    #[test]
    fn expanded_range_allows_tools_but_preserves_other_exclusions() {
        let tool = || WindowProperties {
            tool: true,
            ..main_window()
        };
        assert!(!eligible(&tool(), false));
        assert!(eligible(&tool(), true));
        assert!(eligible(
            &WindowProperties {
                visible: false,
                title: false,
                ..tool()
            },
            true
        ));
        assert!(!eligible(
            &WindowProperties {
                owned: true,
                appwindow: true,
                ..tool()
            },
            false
        ));
        assert!(eligible(
            &WindowProperties {
                owned: true,
                appwindow: true,
                ..tool()
            },
            true
        ));
        for window in [
            WindowProperties {
                own: true,
                ..tool()
            },
            WindowProperties {
                owned: true,
                ..tool()
            },
            WindowProperties {
                noactivate: true,
                ..tool()
            },
            WindowProperties {
                system_shell: true,
                ..tool()
            },
        ] {
            assert!(!eligible(&window, false));
            assert!(!eligible(&window, true));
        }
    }
    #[test]
    fn display_status_distinguishes_minimized_from_hidden() {
        assert_eq!(display_status(true, false, false), WindowStatus::Visible);
        assert_eq!(display_status(true, true, false), WindowStatus::Minimized);
        assert_eq!(display_status(false, false, false), WindowStatus::Hidden);
        assert_eq!(display_status(false, true, false), WindowStatus::Hidden);
        assert_eq!(display_status(true, false, true), WindowStatus::Hidden);
    }
}

#[derive(Default)]
struct WindowProperties {
    visible: bool,
    own: bool,
    owned: bool,
    appwindow: bool,
    tool: bool,
    noactivate: bool,
    cloaked: bool,
    title: bool,
    system_shell: bool,
}

fn eligible(window: &WindowProperties, include_hidden: bool) -> bool {
    !window.own
        && !window.system_shell
        && (!window.owned || window.appwindow)
        && (include_hidden || !window.tool)
        && !window.noactivate
        && (include_hidden || (window.visible && !window.cloaked && window.title))
}

fn display_status(visible: bool, minimized: bool, cloaked: bool) -> WindowStatus {
    if !visible || cloaked {
        WindowStatus::Hidden
    } else if minimized {
        WindowStatus::Minimized
    } else {
        WindowStatus::Visible
    }
}

unsafe fn is_cloaked(hwnd: HWND) -> bool {
    let mut cloaked: u32 = 0;
    DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED as u32,
        (&mut cloaked as *mut u32).cast(),
        std::mem::size_of::<u32>() as u32,
    ) >= 0
        && cloaked != 0
}

unsafe fn candidate(hwnd: HWND, include_hidden: bool) -> bool {
    if hwnd == GetDesktopWindow() || hwnd == GetShellWindow() {
        return false;
    }
    let mut pid = 0;
    if GetWindowThreadProcessId(hwnd, &mut pid) == 0 {
        return false;
    }
    let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    let mut class = [0u16; 256];
    let copied = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32);
    let class = String::from_utf16_lossy(&class[..copied.max(0) as usize]);
    eligible(
        &WindowProperties {
            visible: IsWindowVisible(hwnd) != 0,
            own: pid == std::process::id(),
            owned: !GetWindow(hwnd, GW_OWNER).is_null(),
            appwindow: style & WS_EX_APPWINDOW != 0,
            tool: style & WS_EX_TOOLWINDOW != 0,
            noactivate: style & WS_EX_NOACTIVATE != 0,
            cloaked: is_cloaked(hwnd),
            title: GetWindowTextLengthW(hwnd) > 0,
            system_shell: matches!(
                class.as_str(),
                "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
            ),
        },
        include_hidden,
    )
}

fn query_error() -> QueryError {
    if unsafe { GetLastError() } == ERROR_ACCESS_DENIED {
        QueryError::Permission
    } else {
        QueryError::Failed
    }
}

fn inspect(hwnd_value: usize) -> Result<ObservedWindow, QueryError> {
    let hwnd = hwnd_value as HWND;
    unsafe {
        if IsWindow(hwnd) == 0 || hwnd == GetDesktopWindow() || hwnd == GetShellWindow() {
            return Err(QueryError::Gone);
        }
        // Live identity deliberately ignores candidate visibility, title, and helper styles.
        let mut pid = 0;
        let thread = GetWindowThreadProcessId(hwnd, &mut pid);
        let length = GetWindowTextLengthW(hwnd);
        if thread == 0 || pid == 0 || pid == std::process::id() {
            return Err(QueryError::Gone);
        }
        let mut title = vec![0u16; (length.max(0) as usize + 1).min(32768)];
        let copied = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
        title.truncate(copied.max(0) as usize);
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return Err(query_error());
        }
        let process = Process(handle);
        let mut exit = 0;
        if GetExitCodeProcess(process.0, &mut exit) == 0 {
            return Err(query_error());
        }
        if exit != 259 {
            return Err(QueryError::Gone);
        }
        let mut created: FILETIME = std::mem::zeroed();
        let mut exited: FILETIME = std::mem::zeroed();
        let mut kernel: FILETIME = std::mem::zeroed();
        let mut user: FILETIME = std::mem::zeroed();
        if GetProcessTimes(process.0, &mut created, &mut exited, &mut kernel, &mut user) == 0 {
            return Err(query_error());
        }
        let mut image = vec![0u16; 32768];
        let mut size = image.len() as u32;
        if QueryFullProcessImageNameW(process.0, 0, image.as_mut_ptr(), &mut size) == 0 {
            return Err(query_error());
        }
        image.truncate(size as usize);
        let image = OsString::from_wide(&image);
        let exe = Path::new(&image)
            .file_name()
            .ok_or(QueryError::Failed)?
            .to_string_lossy()
            .into_owned();
        let mut verified_pid = 0;
        if IsWindow(hwnd) == 0
            || GetWindowThreadProcessId(hwnd, &mut verified_pid) != thread
            || verified_pid != pid
        {
            return Err(QueryError::Gone);
        }
        Ok(ObservedWindow {
            identity: Identity {
                hwnd: hwnd_value,
                pid,
                created: ((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64,
                thread,
            },
            title: String::from_utf16_lossy(&title),
            exe,
            status: display_status(
                IsWindowVisible(hwnd) != 0,
                IsIconic(hwnd) != 0,
                is_cloaked(hwnd),
            ),
        })
    }
}

impl WindowQuery for NativeQuery {
    fn enumerate(&self, include_hidden: bool) -> Result<Enumeration, QueryError> {
        struct Context {
            result: Enumeration,
            include_hidden: bool,
        }
        unsafe extern "system" fn collect(hwnd: HWND, parameter: LPARAM) -> i32 {
            let context = &mut *(parameter as *mut Context);
            if !candidate(hwnd, context.include_hidden) {
                return 1;
            }
            match inspect(hwnd as usize) {
                Ok(window) => context.result.windows.push(window),
                Err(QueryError::Permission | QueryError::Failed) => {
                    context.result.inaccessible += 1
                }
                Err(QueryError::Gone) => {}
            }
            1
        }
        let mut context = Context {
            include_hidden,
            result: Enumeration {
                windows: vec![],
                inaccessible: 0,
            },
        };
        if unsafe { EnumWindows(Some(collect), (&mut context as *mut Context) as LPARAM) } == 0 {
            return Err(QueryError::Failed);
        }
        context.result.windows.sort_by(|a, b| {
            a.exe
                .to_lowercase()
                .cmp(&b.exe.to_lowercase())
                .then(a.identity.pid.cmp(&b.identity.pid))
                .then(a.title.cmp(&b.title))
                .then(a.identity.hwnd.cmp(&b.identity.hwnd))
        });
        Ok(context.result)
    }
    fn inspect(&self, hwnd: usize) -> Result<ObservedWindow, QueryError> {
        inspect(hwnd)
    }
}
