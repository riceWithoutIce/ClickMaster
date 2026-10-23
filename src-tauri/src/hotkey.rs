//! Registration and message dispatch share one owned thread. Tests use a fake port.
use crate::ShellState;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tauri::Manager;
use windows_sys::Win32::UI::{
    Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_NOREPEAT},
    WindowsAndMessaging::{PeekMessageW, MSG, PM_REMOVE, WM_HOTKEY},
};
const ID: i32 = 0x434D;
trait Port: Send + 'static {
    fn register(&mut self) -> bool;
    fn pressed(&mut self) -> bool;
    fn release(&mut self);
}
struct Native;
impl Port for Native {
    fn register(&mut self) -> bool {
        unsafe { RegisterHotKey(std::ptr::null_mut(), ID, MOD_NOREPEAT, 0x77) != 0 }
    }
    fn pressed(&mut self) -> bool {
        unsafe {
            let mut message: MSG = std::mem::zeroed();
            let mut pressed = false;
            while PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                pressed |= message.message == WM_HOTKEY && message.wParam == ID as usize;
            }
            pressed
        }
    }
    fn release(&mut self) {
        unsafe {
            UnregisterHotKey(std::ptr::null_mut(), ID);
        }
    }
}
pub struct Runtime {
    stop: Arc<AtomicBool>,
    armed: Arc<AtomicBool>,
    live: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}
impl Runtime {
    pub fn start(app: tauri::AppHandle) -> Result<Self, String> {
        let reader = app.clone();
        Self::with_port(
            Native,
            move || {
                let state = reader.state::<ShellState>();
                state
                    .input
                    .lock()
                    .map(|engine| engine.status.busy && engine.status.phase != "cleanup-failed")
                    .unwrap_or(false)
            },
            {
                let app = app.clone();
                move || {
                    app.state::<ShellState>()
                        .stop_requested
                        .store(true, Ordering::Release);
                }
            },
        )
    }
    fn with_port(
        mut port: impl Port,
        busy: impl Fn() -> bool + Send + 'static,
        pressed: impl Fn() + Send + 'static,
    ) -> Result<Self, String> {
        let stop = Arc::new(AtomicBool::new(false));
        let armed = Arc::new(AtomicBool::new(false));
        let live = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::sync_channel(1);
        let (end, ready, alive) = (stop.clone(), armed.clone(), live.clone());
        let join = thread::Builder::new()
            .name("global-stop-f8".into())
            .spawn(move || {
                if !port.register() {
                    let _ = tx.send(false);
                    return;
                }
                alive.store(true, Ordering::Release);
                if tx.send(true).is_ok() {
                    while !end.load(Ordering::Acquire) {
                        if port.pressed() {
                            pressed();
                        }
                        if ready.load(Ordering::Acquire) && !busy() {
                            break;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                }
                port.release();
                alive.store(false, Ordering::Release);
            })
            .map_err(|_| "全局停止服务无法启动，禁止执行。")?;
        let mut runtime = Self {
            stop,
            armed,
            live,
            join: Some(join),
        };
        if rx.recv_timeout(Duration::from_secs(2)) != Ok(true) {
            runtime.shutdown();
            return Err("无法注册 F8（可能已被其他程序占用），本次未开始。".into());
        }
        Ok(runtime)
    }
    pub fn arm(&self) {
        self.armed.store(true, Ordering::Release);
    }
    pub fn live(&self) -> bool {
        self.live.load(Ordering::Acquire)
            && self
                .join
                .as_ref()
                .is_some_and(|thread| !thread.is_finished())
    }
    pub fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.shutdown();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Fake {
        success: bool,
        press: bool,
        released: Arc<AtomicBool>,
    }
    impl Port for Fake {
        fn register(&mut self) -> bool {
            self.success
        }
        fn pressed(&mut self) -> bool {
            std::mem::take(&mut self.press)
        }
        fn release(&mut self) {
            self.released.store(true, Ordering::Release);
        }
    }
    #[test]
    fn registration_conflict_prevents_start_and_success_dispatches_stop_and_releases() {
        let released = Arc::new(AtomicBool::new(false));
        assert!(Runtime::with_port(
            Fake {
                success: false,
                press: false,
                released: released.clone()
            },
            || true,
            || panic!("conflict")
        )
        .is_err());
        assert!(!released.load(Ordering::Acquire));
        let stopped = Arc::new(AtomicBool::new(false));
        let flag = stopped.clone();
        let mut runtime = Runtime::with_port(
            Fake {
                success: true,
                press: true,
                released: released.clone(),
            },
            || false,
            move || flag.store(true, Ordering::Release),
        )
        .unwrap();
        assert!(runtime.live());
        runtime.arm();
        let deadline = std::time::Instant::now();
        while !stopped.load(Ordering::Acquire) && deadline.elapsed() < Duration::from_secs(1) {
            thread::sleep(Duration::from_millis(2));
        }
        runtime.shutdown();
        assert!(stopped.load(Ordering::Acquire));
        assert!(released.load(Ordering::Acquire)); // Same-thread release, no real F8 registration.
                                                   // Dispatch explicitly tested without scheduling a physical desktop key.
        let mut port = Fake {
            success: true,
            press: true,
            released,
        };
        assert!(port.pressed());
        assert!(!port.pressed());
    }
    #[test]
    fn normal_terminal_releases_registration_without_shutdown_request() {
        let released = Arc::new(AtomicBool::new(false));
        let mut runtime = Runtime::with_port(
            Fake {
                success: true,
                press: false,
                released: released.clone(),
            },
            || false,
            || {},
        )
        .unwrap();
        runtime.arm();
        let start = std::time::Instant::now();
        while !released.load(Ordering::Acquire) && start.elapsed() < Duration::from_secs(1) {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(released.load(Ordering::Acquire));
        assert!(!runtime.live());
        runtime.shutdown();
    }
}
