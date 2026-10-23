//! One worker ticks the single in-memory task. Lock order is always input -> targets.
use crate::{input_native::NativeAdapter, ShellState};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};
pub struct Runtime {
    stopping: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Runtime {
    pub fn start(app: tauri::AppHandle) -> Result<Self, String> {
        let stopping = Arc::new(AtomicBool::new(false));
        let stop = stopping.clone();
        let thread = thread::Builder::new()
            .name("single-input".into())
            .spawn(move || {
                let mut published_revision = u64::MAX;
                while !stop.load(Ordering::Relaxed) {
                    let state = app.state::<ShellState>();
                    let hotkey_failed = state
                        .hotkey
                        .lock()
                        .map(|service| service.as_ref().is_some_and(|service| !service.live()))
                        .unwrap_or(true);
                    let busy = state
                        .input
                        .lock()
                        .map(|engine| engine.status.busy)
                        .unwrap_or(false);
                    if busy && (state.stop_requested.swap(false, Ordering::AcqRel) || hotkey_failed)
                    {
                        let _ = crate::cancel_input_reason(&state, &app, hotkey_failed);
                    }
                    if let Ok(mut input) = state.input.lock() {
                        if input.status.busy && input.status.channel == crate::channels::MESSAGE {
                            if let Ok(mut targets) = state.targets.lock() {
                                let revision = input.status.revision;
                                let binding_revision = targets.status().revision;
                                let mut adapter = NativeAdapter {
                                    targets: &mut targets,
                                };
                                if state.stopping.load(Ordering::Relaxed) {
                                    input.cancel(&mut adapter);
                                } else {
                                    input.tick_with_clock(
                                        elapsed(state.input_origin),
                                        &mut adapter,
                                        crate::ipc_native::random_u32,
                                        || elapsed(state.input_origin),
                                    );
                                }
                                let _changed = input.status.revision != revision;
                                if targets.status().revision != binding_revision {
                                    let _ = app.emit("binding-changed", targets.status());
                                }
                            }
                        }
                    }
                    let status = state.input.lock().ok().map(|mut engine| {
                        let before = engine.status.timing.clone();
                        engine.status.timing.refresh(elapsed(state.input_origin));
                        if engine.status.timing != before {
                            engine.status.revision += 1;
                        }
                        engine.status.clone()
                    });
                    if let Some(status) = status {
                        if !status.busy {
                            if let Ok(_gate) = state.execution_gate.try_lock() {
                                if state
                                    .input
                                    .lock()
                                    .map(|engine| !engine.status.busy)
                                    .unwrap_or(false)
                                {
                                    crate::finish_hotkey(&state);
                                }
                            }
                        }
                        if published_revision != status.revision {
                            crate::publish_input(&app, &status);
                            published_revision = status.revision;
                        }
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            })
            .map_err(|_| "无法启动单次输入服务；不会发送输入。")?;
        Ok(Self {
            stopping,
            thread: Some(thread),
        })
    }
    pub fn shutdown(&mut self) {
        self.stopping.store(true, Ordering::Relaxed);
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
pub fn elapsed(origin: Instant) -> u64 {
    origin.elapsed().as_millis().min(u64::MAX as u128) as u64
}
