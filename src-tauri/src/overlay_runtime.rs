//! Command/acknowledgement boundary: all native resources live and die on one thread.
use crate::{
    overlay::{Binding, Engine, Observation, Position, Status},
    overlay_native::{self, DpiContext, NativeRenderer},
    windows_query::NativeQuery,
    ShellState,
};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};
use tauri::{Emitter, Manager};

enum Command {
    Configure(Binding, Position, Sender<Status>),
    Stop(Sender<Status>),
    Status(Sender<Status>),
    Shutdown,
}

pub struct Runtime {
    sender: Sender<Command>,
    thread: Option<JoinHandle<()>>,
}
impl Runtime {
    pub fn start(app: tauri::AppHandle) -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("coordinate-preview".into())
            .spawn(move || run(app, receiver))
            .map_err(|_| "无法启动坐标预览线程；主界面仍可使用。")?;
        Ok(Self {
            sender,
            thread: Some(thread),
        })
    }
    fn call(&self, command: impl FnOnce(Sender<Status>) -> Command) -> Result<Status, String> {
        let (reply, response) = mpsc::channel();
        self.sender
            .send(command(reply))
            .map_err(|_| "坐标预览服务不可用，请重新启动应用。")?;
        response
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| "坐标预览服务响应超时，请关闭预览或重新启动应用。".into())
    }
    pub fn configure(&self, binding: Binding, position: Position) -> Result<Status, String> {
        self.call(|reply| Command::Configure(binding, position, reply))
    }
    pub fn stop(&self) -> Result<Status, String> {
        self.call(Command::Stop)
    }
    pub fn status(&self) -> Result<Status, String> {
        self.call(Command::Status)
    }
    pub fn shutdown(&mut self) {
        let _ = self.sender.send(Command::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn current_binding(app: &tauri::AppHandle) -> Option<Binding> {
    let state = app.state::<ShellState>();
    let mut targets = state.targets.lock().ok()?;
    if targets.validate(&NativeQuery) {
        let _ = app.emit("binding-changed", targets.status());
    }
    targets.bound_identity()
}

fn run(app: tauri::AppHandle, receiver: Receiver<Command>) {
    let mut engine = Engine::default();
    let mut renderer = NativeRenderer::default();
    // Set only this thread's context, before creating the native window; restore it on exit.
    let context = DpiContext::enter();
    let mut published = engine.status.clone();
    loop {
        let mut reply = None;
        match receiver.recv_timeout(Duration::from_millis(80)) {
            Ok(Command::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(Command::Configure(binding, position, response)) => {
                if context.is_err() {
                    engine.stop(&mut renderer, "无法启用原生预览的 DPI 上下文，预览已关闭。");
                } else {
                    engine.configure(binding, position);
                }
                reply = Some(response);
            }
            Ok(Command::Stop(response)) => {
                engine.stop(&mut renderer, "坐标预览已关闭。");
                reply = Some(response);
            }
            Ok(Command::Status(response)) => {
                reply = Some(response);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if let Some(requested) = engine.requested_binding().cloned() {
            let current = current_binding(&app);
            renderer.target = requested.0.hwnd;
            let observation = if current.as_ref() == Some(&requested) {
                overlay_native::observe(current)
            } else {
                Observation {
                    binding: current,
                    foreground: false,
                    visible: false,
                    minimized: true,
                    cloaked: true,
                    geometry: None,
                }
            };
            engine.tick(observation, &mut renderer);
        }
        overlay_native::pump();
        if renderer.paint_failed() {
            engine.stop(
                &mut renderer,
                "原生预览绘制失败，已关闭；可重试，不影响主界面。",
            );
        }
        if engine.status != published {
            published = engine.status.clone();
            let _ = app.emit("overlay-changed", &published);
        }
        if let Some(reply) = reply {
            let _ = reply.send(engine.status.clone());
        }
    }
    engine.stop(&mut renderer, "坐标预览已关闭。");
    // renderer is closed before the DPI guard is dropped and before this thread terminates.
}
