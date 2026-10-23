//! Only production adapter invokes Windows input APIs. Tests inject fake adapters.
use crate::{
    input::{self, Action, Adapter, Binding, Failure, Message, Pair, Request},
    routing_native::NativeRouting,
    targets::TargetState,
    windows_query::NativeQuery,
};
use windows_sys::Win32::{
    Foundation::{GetLastError, SetLastError, HWND, POINT, RECT},
    Graphics::Gdi::{ClientToScreen, ScreenToClient},
    UI::{
        HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        Input::KeyboardAndMouse::{GetKeyboardLayout, MapVirtualKeyExW, MAPVK_VK_TO_VSC_EX},
        WindowsAndMessaging::{GetClientRect, IsIconic, PostMessageW},
    },
};

pub struct NativeAdapter<'a> {
    pub targets: &'a mut TargetState,
}
impl Adapter for NativeAdapter<'_> {
    fn valid(&mut self, expected: &Binding) -> bool {
        self.targets.validate(&NativeQuery);
        self.targets.validate_routing(&NativeRouting);
        self.targets.routing.snapshot().as_ref() == Some(expected)
    }
    fn prepare(
        &mut self,
        binding: &Binding,
        action: &Action,
        request: &Request,
    ) -> Result<Pair, Failure> {
        let root = binding.root.0.hwnd as HWND;
        let receiver = binding.receiver.hwnd as HWND;
        unsafe {
            if IsIconic(root) != 0 || IsIconic(receiver) != 0 {
                return Err(Failure::reason("目标已最小化，未发送；不承诺最小化兼容。"));
            }
            if let Action::Key(vk) = action {
                for hwnd in [root, receiver] {
                    let (width, height) = crate::routing_native::dimensions(hwnd)
                        .map_err(|_| Failure::reason("无法读取目标客户区。"))?;
                    if width <= 0 || height <= 0 {
                        return Err(Failure::reason("目标客户区不可用，未发送。"));
                    }
                }
                let layout = GetKeyboardLayout(binding.receiver.thread);
                if layout.is_null() {
                    return Err(Failure::reason("无法读取接收窗口键盘布局，未发送。"));
                }
                return input::keyboard_pair(
                    *vk,
                    MapVirtualKeyExW(*vk, MAPVK_VK_TO_VSC_EX, layout),
                );
            }
            let previous = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            if previous.is_null() {
                return Err(Failure::reason("无法确定客户区像素坐标，未发送。"));
            }
            let result = (|| {
                let size = |hwnd| -> Result<(i32, i32), Failure> {
                    let mut rect: RECT = std::mem::zeroed();
                    SetLastError(0);
                    if GetClientRect(hwnd, &mut rect) == 0 {
                        return Err(Failure {
                            reason: "无法读取客户区，未发送。".into(),
                            error_code: Some(GetLastError()),
                        });
                    }
                    Ok((
                        rect.right
                            .checked_sub(rect.left)
                            .ok_or_else(|| Failure::reason("客户区尺寸溢出。"))?,
                        rect.bottom
                            .checked_sub(rect.top)
                            .ok_or_else(|| Failure::reason("客户区尺寸溢出。"))?,
                    ))
                };
                let (width, height) = size(root)?;
                let (receive_width, receive_height) = size(receiver)?;
                input::routed_mouse_pair(
                    *action == Action::Right,
                    (width, height),
                    (receive_width, receive_height),
                    request,
                    |(x, y)| {
                        let mut point = POINT { x, y };
                        // Same awareness context for both coordinate spaces; no cursor movement.
                        if receiver != root {
                            SetLastError(0);
                            if ClientToScreen(root, &mut point) == 0
                                || ScreenToClient(receiver, &mut point) == 0
                            {
                                return Err(Failure {
                                    reason: "无法转换到接收客户区，未发送。".into(),
                                    error_code: Some(GetLastError()),
                                });
                            }
                        }
                        Ok((point.x, point.y))
                    },
                )
            })();
            SetThreadDpiAwarenessContext(previous);
            result
        }
    }
    fn post(&mut self, binding: &Binding, message: Message) -> Result<(), Failure> {
        unsafe {
            SetLastError(0);
            if PostMessageW(
                binding.receiver.hwnd as HWND,
                message.id,
                message.wparam,
                message.lparam,
            ) == 0
            {
                Err(Failure {
                    reason: "PostMessage投递失败；不会自动重试或提升权限。".into(),
                    error_code: Some(GetLastError()),
                })
            } else {
                Ok(())
            }
        }
    }
}
