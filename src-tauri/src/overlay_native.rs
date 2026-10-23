//! Native window and GDI adapter. Only the dedicated preview thread calls these APIs.
use crate::overlay::{Binding, Frame, Geometry, Observation, Renderer};
use std::{
    mem::zeroed,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::{
        Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
        Gdi::*,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::{HiDpi::*, WindowsAndMessaging::*},
};

pub struct DpiContext(isize);
impl DpiContext {
    pub fn enter() -> Result<Self, ()> {
        let previous =
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        if previous.is_null() {
            Err(())
        } else {
            Ok(Self(previous as isize))
        }
    }
}
impl Drop for DpiContext {
    fn drop(&mut self) {
        unsafe {
            SetThreadDpiAwarenessContext(self.0 as _);
        }
    }
}

unsafe fn eligible_foreground(hwnd: HWND) -> bool {
    let mut cloaked: u32 = 0;
    let result = DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED as u32,
        (&mut cloaked as *mut u32).cast(),
        size_of::<u32>() as u32,
    );
    GetForegroundWindow() == hwnd
        && IsWindowVisible(hwnd) != 0
        && IsIconic(hwnd) == 0
        && result >= 0
        && cloaked == 0
}

pub fn observe(binding: Option<Binding>) -> Observation {
    let mut result = Observation {
        binding,
        foreground: false,
        visible: false,
        minimized: true,
        cloaked: true,
        geometry: None,
    };
    let Some((identity, _)) = &result.binding else {
        return result;
    };
    let hwnd = identity.hwnd as HWND;
    unsafe {
        result.foreground = GetForegroundWindow() == hwnd;
        result.visible = IsWindowVisible(hwnd) != 0;
        result.minimized = IsIconic(hwnd) != 0;
        let mut cloaked: u32 = 0;
        let queried = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED as u32,
            (&mut cloaked as *mut u32).cast(),
            size_of::<u32>() as u32,
        );
        result.cloaked = queried < 0 || cloaked != 0;
        if !result.foreground || !result.visible || result.minimized || result.cloaked {
            return result;
        }
        let mut client: RECT = zeroed();
        if GetClientRect(hwnd, &mut client) == 0 {
            return result;
        }
        let mut origin = POINT {
            x: client.left,
            y: client.top,
        };
        let mut end = POINT {
            x: client.right,
            y: client.bottom,
        };
        if ClientToScreen(hwnd, &mut origin) == 0 || ClientToScreen(hwnd, &mut end) == 0 {
            return result;
        }
        // PMv2 thread: both client corners are mapped to physical screen pixels. No second DPI multiplication.
        result.geometry = end
            .x
            .checked_sub(origin.x)
            .zip(end.y.checked_sub(origin.y))
            .map(|(width, height)| Geometry {
                left: origin.x,
                top: origin.y,
                width,
                height,
                dpi: GetDpiForWindow(hwnd).max(96),
            });
    }
    result
}

#[derive(Default)]
struct PaintData {
    frame: Option<Frame>,
    failed: bool,
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = &*(lparam as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    match message {
        WM_NCHITTEST => HTTRANSPARENT as LRESULT,
        WM_MOUSEACTIVATE => MA_NOACTIVATE as LRESULT,
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            let data = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PaintData;
            if !data.is_null() {
                let mut paint: PAINTSTRUCT = zeroed();
                let dc = BeginPaint(hwnd, &mut paint);
                if dc.is_null() {
                    (*data).failed = true;
                } else if let Some(frame) = &(*data).frame {
                    if draw(dc, frame).is_err() {
                        (*data).failed = true;
                    }
                }
                EndPaint(hwnd, &paint);
            }
            0
        }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

// Restores the old selected object before deleting an owned GDI object, including failure paths.
struct Selected {
    dc: HDC,
    object: HGDIOBJ,
    previous: HGDIOBJ,
    owned: bool,
}
impl Selected {
    unsafe fn new(dc: HDC, object: HGDIOBJ, owned: bool) -> Result<Self, ()> {
        if object.is_null() {
            return Err(());
        }
        let previous = SelectObject(dc, object);
        if previous.is_null() || previous as isize == -1 {
            if owned {
                DeleteObject(object);
            }
            Err(())
        } else {
            Ok(Self {
                dc,
                object,
                previous,
                owned,
            })
        }
    }
}
impl Drop for Selected {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            if self.owned {
                DeleteObject(self.object);
            }
        }
    }
}

unsafe fn draw(dc: HDC, frame: &Frame) -> Result<(), ()> {
    let g = frame.geometry;
    let bounds = RECT {
        left: 0,
        top: 0,
        right: g.width,
        bottom: g.height,
    };
    if FillRect(dc, &bounds, GetStockObject(BLACK_BRUSH) as HBRUSH) == 0 {
        return Err(());
    }
    let _pen = Selected::new(dc, CreatePen(PS_SOLID, 1, 0x00e0b030), true)?;
    let _brush = Selected::new(dc, GetStockObject(HOLLOW_BRUSH), false)?;
    if Rectangle(dc, 0, 0, g.width, g.height) == 0 {
        return Err(());
    }
    if let Some((x, y)) = frame.point {
        let radius = (8 * g.dpi as i64 / 96).clamp(6, 24) as i32;
        if MoveToEx(dc, x.saturating_sub(radius).max(0), y, null_mut()) == 0
            || LineTo(dc, x.saturating_add(radius).min(g.width - 1), y) == 0
            || MoveToEx(dc, x, y.saturating_sub(radius).max(0), null_mut()) == 0
            || LineTo(dc, x, y.saturating_add(radius).min(g.height - 1)) == 0
        {
            return Err(());
        }
    }
    let face: Vec<u16> = "Segoe UI".encode_utf16().chain(Some(0)).collect();
    let font_height = -(14 * g.dpi as i64 / 96).clamp(12, 48) as i32;
    let _font = Selected::new(
        dc,
        CreateFontW(
            font_height,
            0,
            0,
            0,
            500,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            0,
            0,
            NONANTIALIASED_QUALITY as u32,
            0,
            face.as_ptr(),
        ),
        true,
    )?;
    if SetBkMode(dc, TRANSPARENT as i32) == 0 || SetTextColor(dc, 0x00e0b030) == u32::MAX {
        return Err(());
    }
    let text: Vec<u16> = frame.label.encode_utf16().collect();
    if TextOutW(dc, 8, 6, text.as_ptr(), text.len() as i32) == 0 {
        return Err(());
    }
    Ok(())
}

struct NativeWindow {
    hwnd: HWND,
    class: Vec<u16>,
    data: Box<PaintData>,
}
impl NativeWindow {
    fn create() -> Result<Self, ()> {
        unsafe {
            let class: Vec<u16> = "ClickMaster.CoordinateOverlay.POC"
                .encode_utf16()
                .chain(Some(0))
                .collect();
            let instance = GetModuleHandleW(null());
            if instance.is_null() {
                return Err(());
            }
            let definition = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                lpszClassName: class.as_ptr(),
                ..zeroed()
            };
            if RegisterClassW(&definition) == 0 {
                return Err(());
            }
            let mut data = Box::<PaintData>::default();
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                class.as_ptr(),
                class.as_ptr(),
                WS_POPUP,
                0,
                0,
                1,
                1,
                null_mut(),
                null_mut(),
                instance,
                (&mut *data as *mut PaintData).cast(),
            );
            if hwnd.is_null() {
                UnregisterClassW(class.as_ptr(), instance);
                return Err(());
            }
            let window = Self { hwnd, class, data };
            if SetLayeredWindowAttributes(hwnd, 0, 255, LWA_COLORKEY) == 0 {
                return Err(());
            }
            Ok(window)
        }
    }
}
impl Drop for NativeWindow {
    fn drop(&mut self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
            DestroyWindow(self.hwnd);
            UnregisterClassW(self.class.as_ptr(), GetModuleHandleW(null()));
        }
    }
}

#[derive(Default)]
pub struct NativeRenderer {
    window: Option<NativeWindow>,
    pub target: usize,
}
impl NativeRenderer {
    pub fn paint_failed(&self) -> bool {
        self.window
            .as_ref()
            .is_some_and(|window| window.data.failed)
    }
}
impl Renderer for NativeRenderer {
    fn show(&mut self, frame: &Frame) -> Result<bool, ()> {
        unsafe {
            if !eligible_foreground(self.target as HWND) {
                self.hide()?;
                return Ok(false);
            }
            if self.window.is_none() {
                self.window = Some(NativeWindow::create()?);
            }
            let window = self.window.as_mut().unwrap();
            let g = frame.geometry;
            window.data.frame = Some(frame.clone());
            window.data.failed = false;
            if SetWindowPos(
                window.hwnd,
                HWND_TOPMOST,
                g.left,
                g.top,
                g.width,
                g.height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            ) == 0
            {
                return Err(());
            }
            if InvalidateRect(window.hwnd, null(), 0) == 0 {
                return Err(());
            }
            if let Some(frame) = window.data.frame.as_mut() {
                frame.geometry.dpi = GetDpiForWindow(window.hwnd).max(96);
            }
            UpdateWindow(window.hwnd);
            if window.data.failed {
                return Err(());
            }
            // Close the query/show race without ever activating the target.
            if !eligible_foreground(self.target as HWND) {
                self.hide()?;
                return Ok(false);
            }
            Ok(true)
        }
    }
    fn hide(&mut self) -> Result<(), ()> {
        if let Some(window) = &self.window {
            unsafe {
                ShowWindow(window.hwnd, SW_HIDE);
                if IsWindowVisible(window.hwnd) != 0 {
                    return Err(());
                }
            }
        }
        Ok(())
    }
    fn close(&mut self) {
        self.window = None;
    }
}

pub fn pump() {
    unsafe {
        let mut message: MSG = zeroed();
        while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}
