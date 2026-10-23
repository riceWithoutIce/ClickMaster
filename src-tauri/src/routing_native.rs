//! Read-only receiver discovery for the explicitly supported host/CoreWindow structure.
use crate::{
    routing::{Discovery, Query, Receiver},
    targets::{Identity, WindowQuery},
    windows_query::NativeQuery,
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
    UI::{
        HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        WindowsAndMessaging::{
            EnumChildWindows, GetAncestor, GetClassNameW, GetClientRect, IsChild, IsWindowVisible,
            GA_ROOT,
        },
    },
};

pub struct NativeRouting;
fn class(hwnd: HWND) -> Result<String, ()> {
    let mut value = [0u16; 256];
    let count = unsafe { GetClassNameW(hwnd, value.as_mut_ptr(), value.len() as i32) };
    if count <= 0 {
        return Err(());
    }
    Ok(String::from_utf16_lossy(&value[..count as usize]))
}
pub fn dimensions(hwnd: HWND) -> Result<(i32, i32), ()> {
    unsafe {
        let previous = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        if previous.is_null() {
            return Err(());
        }
        let mut rect: RECT = std::mem::zeroed();
        let success = GetClientRect(hwnd, &mut rect);
        SetThreadDpiAwarenessContext(previous);
        if success == 0 {
            return Err(());
        }
        Ok((
            rect.right.checked_sub(rect.left).ok_or(())?,
            rect.bottom.checked_sub(rect.top).ok_or(())?,
        ))
    }
}
impl Query for NativeRouting {
    fn inspect(&self, root: &Identity, receiver: &Identity) -> Result<Receiver, ()> {
        let bound = NativeQuery.inspect(root.hwnd).map_err(|_| ())?;
        if bound.identity != *root {
            return Err(());
        }
        let observed = NativeQuery.inspect(receiver.hwnd).map_err(|_| ())?;
        if observed.identity != *receiver {
            return Err(());
        }
        let hwnd = receiver.hwnd as HWND;
        let kind = class(hwnd)?;
        if receiver != root
            && (kind != "Windows.UI.Core.CoreWindow"
                || class(root.hwnd as HWND)? != "ApplicationFrameWindow"
                || unsafe {
                    IsChild(root.hwnd as HWND, hwnd) == 0
                        || GetAncestor(hwnd, GA_ROOT) != root.hwnd as HWND
                })
        {
            return Err(());
        }
        let (width, height) = dimensions(hwnd)?;
        Ok(Receiver {
            identity: observed.identity,
            exe: observed.exe,
            class: kind,
            width,
            height,
        })
    }
    fn discover(&self, root: &Identity) -> Result<Discovery, ()> {
        let observed = self.inspect(root, root)?;
        let hosted = observed.class == "ApplicationFrameWindow";
        let mut contents = vec![];
        if hosted {
            struct Context<'a> {
                root: &'a Identity,
                contents: Vec<Receiver>,
                failed: bool,
            }
            unsafe extern "system" fn collect(hwnd: HWND, parameter: LPARAM) -> i32 {
                let context = &mut *(parameter as *mut Context);
                let kind = match class(hwnd) {
                    Ok(kind) => kind,
                    Err(_) => {
                        context.failed = true;
                        return 1;
                    }
                };
                if kind != "Windows.UI.Core.CoreWindow" || IsWindowVisible(hwnd) == 0 {
                    return 1;
                }
                let mut cloaked = 0u32;
                if DwmGetWindowAttribute(
                    hwnd,
                    DWMWA_CLOAKED as u32,
                    (&mut cloaked as *mut u32).cast(),
                    4,
                ) < 0
                {
                    context.failed = true;
                    return 1;
                }
                if cloaked != 0 {
                    return 1;
                }
                let result = NativeQuery
                    .inspect(hwnd as usize)
                    .map_err(|_| ())
                    .and_then(|item| NativeRouting.inspect(context.root, &item.identity));
                match result {
                    Ok(item) if item.width > 0 && item.height > 0 => context.contents.push(item),
                    Ok(_) => {}
                    Err(_) => context.failed = true,
                }
                1
            }
            let mut context = Context {
                root,
                contents: vec![],
                failed: false,
            };
            // EnumChildWindows returns zero even for a valid empty child tree; validate the root separately.
            unsafe {
                EnumChildWindows(
                    root.hwnd as HWND,
                    Some(collect),
                    (&mut context as *mut Context) as LPARAM,
                );
            }
            if context.failed {
                return Err(());
            }
            self.inspect(root, root)?;
            contents = context.contents;
        }
        Ok(Discovery {
            root: observed,
            hosted,
            contents,
        })
    }
}
