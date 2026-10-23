mod channels;
mod duration;
mod hotkey;
mod input;
mod input_native;
mod input_runtime;
mod ipc;
mod ipc_native;
mod ipc_runtime;
mod ipc_storage;
mod ipc_worker;
mod overlay;
mod overlay_native;
mod overlay_runtime;
mod perf_native;
mod perf_states;
mod perf_webview;
mod performance;
mod repeat;
mod routing;
mod routing_native;
mod settings;
mod targets;
mod window_policy;
mod windows_query;

use serde::Serialize;
use settings::SettingsStore;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

struct ShellState {
    diagnostic_range: Mutex<Option<repeat::Range>>,
    tray_revision: AtomicU64,
    stop_requested: AtomicBool,
    hotkey: Mutex<Option<hotkey::Runtime>>,
    tray_stop: Mutex<Option<MenuItem<tauri::Wry>>>,
    execution_gate: Mutex<()>,
    channel: Mutex<channels::View>,
    ipc_worker: Mutex<Option<ipc_runtime::Runtime>>,
    store: Mutex<SettingsStore>,
    tray_available: AtomicBool,
    close_pending: AtomicBool,
    stopping: AtomicBool,
    targets: Mutex<targets::TargetState>,
    overlay: Mutex<Option<overlay_runtime::Runtime>>,
    input: Mutex<input::Engine>,
    input_worker: Mutex<Option<input_runtime::Runtime>>,
    input_origin: std::time::Instant,
}

fn require_idle(state: &ShellState) -> Result<(), String> {
    channels::idle(
        state
            .input
            .lock()
            .map_err(|_| "输入状态不可用。")?
            .status
            .busy,
        state.stopping.load(Ordering::Acquire),
    )
}
fn channel_view(state: &ShellState, force: bool) -> Result<channels::View, String> {
    let targets = state.targets.lock().map_err(|_| "目标状态不可用。")?;
    let binding = targets.bound_identity();
    let epoch = targets.status().epoch;
    let supported_name = targets
        .status()
        .bound
        .is_some_and(|target| target.exe.eq_ignore_ascii_case("TapTapLoot.exe"));
    drop(targets);
    let mut view = state.channel.lock().map_err(|_| "通道状态不可用。")?;
    channels::refresh(&mut view, epoch, force, || {
        if supported_name {
            binding
                .as_ref()
                .ok_or_else(|| "请重新绑定目标。".to_string())
                .and_then(|(id, _)| {
                    ipc_native::Lease::open(id)
                        .and_then(|lease| lease.validate_for_send())
                        .map_err(String::from)
                })
        } else {
            Err("专用 IPC 仅适用于已验证版本的 TapTapLoot；请绑定对应窗口。".into())
        }
    });
    Ok(view.clone())
}
#[tauri::command]
fn get_channel_status(state: tauri::State<ShellState>) -> Result<channels::View, String> {
    let _gate = state
        .execution_gate
        .lock()
        .map_err(|_| "执行状态不可用。")?;
    channel_view(&state, false)
}
#[tauri::command]
fn select_channel(
    channel: String,
    state: tauri::State<ShellState>,
) -> Result<channels::View, String> {
    let _gate = state
        .execution_gate
        .lock()
        .map_err(|_| "执行状态不可用。")?;
    require_idle(&state)?;
    channel_view(&state, true)?;
    let mut view = state.channel.lock().map_err(|_| "通道状态不可用。")?;
    channels::select(&mut view, &channel)?;
    stop_overlay(&state)?;
    Ok(view.clone())
}

#[tauri::command]
fn start_single_input(
    request: input::Request,
    channel: String,
    expected_epoch: u64,
    expected_route_epoch: u64,
    app: tauri::AppHandle,
    state: tauri::State<ShellState>,
) -> Result<input::Status, String> {
    start_input(
        request,
        channel,
        expected_epoch,
        expected_route_epoch,
        None,
        app,
        &state,
    )
}
#[tauri::command]
fn start_repeat_input(
    request: input::Request,
    channel: String,
    expected_epoch: u64,
    expected_route_epoch: u64,
    interval: repeat::Request,
    duration: duration::Request,
    app: tauri::AppHandle,
    state: tauri::State<ShellState>,
) -> Result<input::Status, String> {
    start_input(
        request,
        channel,
        expected_epoch,
        expected_route_epoch,
        Some((interval, duration)),
        app,
        &state,
    )
}
fn start_input(
    request: input::Request,
    channel: String,
    expected_epoch: u64,
    expected_route_epoch: u64,
    interval: Option<(repeat::Request, duration::Request)>,
    app: tauri::AppHandle,
    state: &ShellState,
) -> Result<input::Status, String> {
    let _gate = state
        .execution_gate
        .lock()
        .map_err(|_| "执行状态不可用。")?;
    require_idle(&state)?;
    let (range, policy) = match interval {
        Some((interval, duration)) => (
            Some(interval.validate(channel == channels::IPC)?),
            duration.validate()?,
        ),
        None => (None, duration::Policy::Permanent),
    };
    if state
        .input_worker
        .lock()
        .map_err(|_| "执行服务不可用。")?
        .is_none()
    {
        return Err("统一停止与执行服务未启动，禁止开始。".into());
    }
    let view = channel_view(&state, true)?;
    if channel != view.selected {
        return Err("执行通道已变化，请重新确认。".into());
    }
    if channel == channels::IPC {
        if !view.ipc_available {
            return Err(view.reason);
        }
        let (identity, epoch) = state
            .targets
            .lock()
            .map_err(|_| "目标状态不可用。")?
            .bound_identity()
            .ok_or("目标已失效，请重新绑定。")?;
        if epoch != expected_epoch {
            return Err("目标绑定已变化。".into());
        }
        let lease = ipc_native::Lease::open(&identity)?;
        stop_overlay(&state)?;
        let mut worker = state.ipc_worker.lock().map_err(|_| "IPC 服务不可用。")?;
        // Reap only our previous completed supervisor before creating a new one.
        if let Some(mut old) = worker.take() {
            old.shutdown();
        }
        let hotkey = prepare_hotkey(state, &app)?;
        {
            let mut engine = state.input.lock().map_err(|_| "输入状态不可用。")?;
            let s = &mut engine.status;
            s.revision += 1;
            s.task_id = s.task_id.checked_add(1).ok_or("任务计数达到安全上限。")?;
            s.repeating = range.is_some();
            s.timing = duration::Clock::new(policy);
            s.completed = 0;
            s.channel = channels::IPC.into();
            s.ipc = Some(ipc::Receipt::default());
            s.phase = "countdown".into();
            s.busy = true;
            s.remaining_ms = input::COUNTDOWN_MS;
            s.message = ipc::phase_message("countdown").into();
            s.summary = if range.is_some() {
                "TapTapLoot 专用 IPC · 持续重复"
            } else {
                "TapTapLoot 专用 IPC · 单次"
            }
            .into();
            s.receiver = None;
            s.down = None;
            s.up = None;
        }
        match ipc_runtime::Runtime::start(app.clone(), lease, range) {
            Ok(runtime) => *worker = Some(runtime),
            Err(error) => {
                let mut engine = state.input.lock().map_err(|_| "输入状态不可用。")?;
                engine.status.busy = false;
                engine.status.phase = "failed".into();
                engine.status.revision += 1;
                engine.status.message = error.clone();
                return Err(error);
            }
        }
        hotkey.arm();
        if let Ok(mut saved) = state.diagnostic_range.lock() {
            *saved = range;
        }
        performance::parameters(range);
        *state.hotkey.lock().map_err(|_| "停止服务不可用。")? = Some(hotkey);
        return Ok(state
            .input
            .lock()
            .map_err(|_| "输入状态不可用。")?
            .status
            .clone());
    }
    if state
        .input_worker
        .lock()
        .map_err(|_| "输入服务不可用。")?
        .is_none()
    {
        return Err("单次输入服务未启动，不会发送输入。".into());
    }
    if state.stopping.load(Ordering::Relaxed) {
        return Err("程序正在退出，不接受输入任务。".into());
    }
    let mut targets = state.targets.lock().map_err(|_| "目标状态不可用。")?;
    if targets.validate(&windows_query::NativeQuery) {
        let _ = app.emit("binding-changed", targets.status());
    }
    targets.validate_routing(&routing_native::NativeRouting);
    let binding = targets
        .routing
        .snapshot()
        .ok_or("请先确认有效输入接收窗口；失效后需重新绑定。")?;
    if binding.root.1 != expected_epoch || binding.epoch != expected_route_epoch {
        return Err("目标绑定已变化，请重新确认后测试。".into());
    }
    input::action(&request)?;
    drop(targets);
    let hotkey = prepare_hotkey(state, &app)?;
    let mut input = state.input.lock().map_err(|_| "输入状态不可用。")?;
    input.start_task(
        input_runtime::elapsed(state.input_origin),
        binding,
        request,
        range,
    )?;
    input.status.timing = duration::Clock::new(policy);
    let status = input.status.clone();
    drop(input);
    hotkey.arm();
    if let Ok(mut saved) = state.diagnostic_range.lock() {
        *saved = range;
    }
    performance::parameters(range);
    *state.hotkey.lock().map_err(|_| "停止服务不可用。")? = Some(hotkey);
    let _ = app.emit("input-changed", &status);
    performance::context(&status);
    Ok(status)
}

fn prepare_hotkey(state: &ShellState, app: &tauri::AppHandle) -> Result<hotkey::Runtime, String> {
    if let Some(mut previous) = state.hotkey.lock().map_err(|_| "停止服务不可用。")?.take()
    {
        previous.shutdown();
    }
    state.stop_requested.store(false, Ordering::Release);
    hotkey::Runtime::start(app.clone())
}
fn finish_hotkey(state: &ShellState) {
    if let Ok(mut service) = state.hotkey.lock() {
        if let Some(mut service) = service.take() {
            service.shutdown();
        }
    }
}
fn publish_input(app: &tauri::AppHandle, status: &input::Status) {
    performance::context(status);
    let _ = app.emit("input-changed", status);
    // Menu/tray setters synchronously dispatch to the UI thread. Never call them
    // while holding scheduler locks: exit can be waiting for the same worker.
    let handle = app.clone();
    let status = status.clone();
    let _ = app.run_on_main_thread(move || {
        let state = handle.state::<ShellState>();
        if state
            .tray_revision
            .fetch_max(status.revision, Ordering::AcqRel)
            > status.revision
        {
            return;
        }
        if let Ok(stop) = state.tray_stop.lock() {
            if let Some(stop) = stop.as_ref() {
                let _ = stop.set_enabled(status.busy);
            }
        }
        if let Some(tray) = handle.tray_by_id("main-tray") {
            let label = if status.phase == "stopping" {
                "停止中"
            } else if status.busy {
                if status.repeating {
                    "重复运行"
                } else {
                    "单次测试"
                }
            } else {
                "待命"
            };
            let _ = tray.set_tooltip(Some(format!(
                "ClickMaster · {label} · 已完成 {}",
                status.completed
            )));
        }
    });
}

fn cancel_input(state: &ShellState, app: &tauri::AppHandle) -> Result<input::Status, String> {
    cancel_input_reason(state, app, false)
}
fn stop_service_failure(status: &mut input::Status) {
    if status.phase == "cancelled" {
        status.phase = "failed".into();
        status.message = "全局停止服务异常，任务已停止。".into();
        status.timing.end_reason = Some("停止服务异常".into());
        status.timing.trigger_reason = None;
        status.revision += 1;
    }
}
fn cancel_input_reason(
    state: &ShellState,
    app: &tauri::AppHandle,
    service_failed: bool,
) -> Result<input::Status, String> {
    let _gate = state
        .execution_gate
        .lock()
        .map_err(|_| "执行状态不可用。")?;
    {
        let mut input = state.input.lock().map_err(|_| "输入状态不可用。")?;
        if input.status.busy && input.status.phase != "cleanup-failed" {
            input.status.phase = "stopping".into();
            input.status.message = "正在停止并清理；已发送的操作无法撤回。".into();
            input.status.revision += 1;
            publish_input(app, &input.status);
        }
    }
    if state
        .input
        .lock()
        .map_err(|_| "输入状态不可用。")?
        .status
        .channel
        == channels::IPC
    {
        if let Some(worker) = state
            .ipc_worker
            .lock()
            .map_err(|_| "IPC 服务不可用。")?
            .as_mut()
        {
            worker.shutdown();
        }
        finish_hotkey(state);
        let mut input = state.input.lock().map_err(|_| "输入状态不可用。")?;
        if service_failed {
            stop_service_failure(&mut input.status);
        }
        publish_input(app, &input.status);
        return Ok(input.status.clone());
    }
    let mut input = state.input.lock().map_err(|_| "输入状态不可用。")?;
    let mut targets = state.targets.lock().map_err(|_| "目标状态不可用。")?;
    input.cancel_at(
        input_runtime::elapsed(state.input_origin),
        &mut input_native::NativeAdapter {
            targets: &mut targets,
        },
    );
    if service_failed {
        stop_service_failure(&mut input.status);
    }
    let status = input.status.clone();
    let _ = app.emit("input-changed", &status);
    drop(targets);
    drop(input);
    finish_hotkey(state);
    publish_input(app, &status);
    Ok(status)
}

#[tauri::command]
fn cancel_single_input(
    app: tauri::AppHandle,
    state: tauri::State<ShellState>,
) -> Result<input::Status, String> {
    cancel_input(&state, &app)
}
#[tauri::command]
fn get_input_status(state: tauri::State<ShellState>) -> Result<input::Status, String> {
    let status = state
        .input
        .lock()
        .map_err(|_| "输入状态不可用。")?
        .status
        .clone();
    performance::context(&status);
    Ok(status)
}

fn stop_overlay(state: &ShellState) -> Result<overlay::Status, String> {
    match state
        .overlay
        .lock()
        .map_err(|_| "预览状态不可用。")?
        .as_ref()
    {
        Some(runtime) => runtime.stop(),
        None => Ok(overlay::Status::default()),
    }
}

#[tauri::command]
fn set_overlay(
    enabled: bool,
    expected_epoch: u64,
    position: overlay::Position,
    state: tauri::State<ShellState>,
) -> Result<overlay::Status, String> {
    let _gate = state
        .execution_gate
        .lock()
        .map_err(|_| "执行状态不可用。")?;
    require_idle(&state)?;
    if enabled
        && state
            .channel
            .lock()
            .map_err(|_| "通道状态不可用。")?
            .selected
            != channels::MESSAGE
    {
        return Err("IPC 通道不使用坐标预览。".into());
    }
    if !enabled {
        return stop_overlay(&state);
    }
    let binding = {
        let mut targets = state.targets.lock().map_err(|_| "目标状态不可用。")?;
        targets.validate(&windows_query::NativeQuery);
        let binding = targets.bound_identity().ok_or("请先绑定有效的目标窗口。")?;
        if binding.1 != expected_epoch {
            return Err("目标绑定已变化，请重新开启预览。".into());
        }
        binding
    };
    state
        .overlay
        .lock()
        .map_err(|_| "预览状态不可用。")?
        .as_ref()
        .ok_or("原生预览服务未启动。")?
        .configure(binding, position)
}

#[tauri::command]
fn get_overlay_status(state: tauri::State<ShellState>) -> Result<overlay::Status, String> {
    state
        .overlay
        .lock()
        .map_err(|_| "预览状态不可用。")?
        .as_ref()
        .ok_or("原生预览服务未启动。")?
        .status()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: String,
    minimize_keep_taskbar: bool,
    tray_available: bool,
    warning: Option<String>,
    close_pending: bool,
}

#[tauri::command]
fn get_app_info(app: tauri::AppHandle, state: tauri::State<ShellState>) -> Result<AppInfo, String> {
    let store = state.store.lock().map_err(|_| "配置状态不可用。")?;
    let tray_available = state.tray_available.load(Ordering::Relaxed);
    let warning = match (&store.warning, tray_available) {
        (Some(message), false) => Some(format!("{message} 托盘不可用，最小化将保留任务栏图标。")),
        (Some(message), true) => Some(message.clone()),
        (None, false) => Some("托盘初始化失败，最小化将保留任务栏图标。".into()),
        _ => None,
    };
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        minimize_keep_taskbar: store.settings.minimize_keep_taskbar,
        tray_available,
        warning,
        close_pending: state.close_pending.load(Ordering::Relaxed),
    })
}

#[tauri::command]
fn refresh_targets(
    include_hidden: bool,
    state: tauri::State<ShellState>,
) -> Result<targets::TargetList, String> {
    let _gate = state
        .execution_gate
        .lock()
        .map_err(|_| "执行状态不可用。")?;
    require_idle(&state)?;
    state
        .targets
        .lock()
        .map_err(|_| "目标状态不可用。")?
        .refresh(&windows_query::NativeQuery, include_hidden)
}

#[tauri::command]
fn bind_target(
    token: String,
    app: tauri::AppHandle,
    state: tauri::State<ShellState>,
) -> Result<targets::BindingStatus, String> {
    let _gate = state
        .execution_gate
        .lock()
        .map_err(|_| "执行状态不可用。")?;
    require_idle(&state)?;
    let mut input = state.input.lock().map_err(|_| "输入状态不可用。")?;
    let mut targets = state.targets.lock().map_err(|_| "目标状态不可用。")?;
    let status = input.cancel(&mut input_native::NativeAdapter {
        targets: &mut targets,
    });
    let _ = app.emit("input-changed", status);
    drop(targets);
    stop_overlay(&state)?;
    let mut targets = state.targets.lock().map_err(|_| "目标状态不可用。")?;
    targets.bind(&token, &windows_query::NativeQuery)?;
    targets.discover_routing(&routing_native::NativeRouting);
    Ok(targets.status())
}

#[tauri::command]
fn choose_receiver(
    token: String,
    expected_epoch: u64,
    app: tauri::AppHandle,
    state: tauri::State<ShellState>,
) -> Result<targets::BindingStatus, String> {
    let _gate = state
        .execution_gate
        .lock()
        .map_err(|_| "执行状态不可用。")?;
    require_idle(&state)?;
    if state
        .channel
        .lock()
        .map_err(|_| "通道状态不可用。")?
        .selected
        != channels::MESSAGE
    {
        return Err("IPC 不使用窗口消息接收路由。".into());
    }
    let mut input = state.input.lock().map_err(|_| "输入状态不可用。")?;
    let mut targets = state.targets.lock().map_err(|_| "目标状态不可用。")?;
    targets.validate(&windows_query::NativeQuery);
    if targets.bound_identity().map(|bound| bound.1) != Some(expected_epoch) {
        return Err("绑定已变化，请重新确认。".into());
    }
    let status = input.cancel(&mut input_native::NativeAdapter {
        targets: &mut targets,
    });
    let _ = app.emit("input-changed", status);
    targets.choose_receiver(&token, &routing_native::NativeRouting)
}

#[tauri::command]
fn unbind_target(
    app: tauri::AppHandle,
    state: tauri::State<ShellState>,
) -> Result<targets::BindingStatus, String> {
    let _gate = state
        .execution_gate
        .lock()
        .map_err(|_| "执行状态不可用。")?;
    require_idle(&state)?;
    let mut input = state.input.lock().map_err(|_| "输入状态不可用。")?;
    let mut targets = state.targets.lock().map_err(|_| "目标状态不可用。")?;
    let status = input.cancel(&mut input_native::NativeAdapter {
        targets: &mut targets,
    });
    let _ = app.emit("input-changed", status);
    drop(targets);
    stop_overlay(&state)?;
    let mut targets = state.targets.lock().map_err(|_| "目标状态不可用。")?;
    Ok(targets.unbind())
}

#[tauri::command]
fn get_binding_status(state: tauri::State<ShellState>) -> Result<targets::BindingStatus, String> {
    Ok(state
        .targets
        .lock()
        .map_err(|_| "目标状态不可用。")?
        .status())
}

impl window_policy::OwnWindow for tauri::WebviewWindow {
    fn taskbar(&self, skip: bool) -> Result<(), ()> {
        self.set_skip_taskbar(skip).map_err(|_| ())
    }
    fn hide(&self) -> Result<(), ()> {
        self.hide().map_err(|_| ())?;
        performance::window_state(self.is_visible().ok(), self.is_minimized().ok());
        Ok(())
    }
    fn show(&self) -> Result<(), ()> {
        self.show().map_err(|_| ())?;
        performance::window_state(self.is_visible().ok(), self.is_minimized().ok());
        Ok(())
    }
    fn minimize(&self) -> Result<(), ()> {
        self.minimize().map_err(|_| ())?;
        performance::window_state(self.is_visible().ok(), self.is_minimized().ok());
        Ok(())
    }
    fn unminimize(&self) -> Result<(), ()> {
        self.unminimize().map_err(|_| ())?;
        performance::window_state(self.is_visible().ok(), self.is_minimized().ok());
        Ok(())
    }
}

#[tauri::command]
fn resolve_close(
    action: String,
    window: tauri::WebviewWindow,
    state: tauri::State<ShellState>,
) -> Result<(), String> {
    match action.as_str() {
        "cancel" => {
            state.close_pending.store(false, Ordering::Relaxed);
            Ok(())
        }
        "exit" => {
            window.app_handle().exit(0);
            Ok(())
        }
        "tuck" => {
            let keep = state
                .store
                .lock()
                .map_err(|_| "配置状态不可用。")?
                .settings
                .minimize_keep_taskbar;
            window_policy::tuck(&window, keep, state.tray_available.load(Ordering::Relaxed))?;
            state.close_pending.store(false, Ordering::Relaxed);
            Ok(())
        }
        _ => Err("未知关闭选项。".into()),
    }
}

#[tauri::command]
fn set_minimize_preference(
    keep_taskbar: bool,
    state: tauri::State<ShellState>,
) -> Result<(), String> {
    let _gate = state
        .execution_gate
        .lock()
        .map_err(|_| "执行状态不可用。")?;
    require_idle(&state)?;
    state
        .store
        .lock()
        .map_err(|_| "配置状态不可用。")?
        .save_preference(keep_taskbar)
}

#[tauri::command]
fn get_performance_status(app: tauri::AppHandle) -> performance::View {
    if performance::active() {
        if let Some(window) = app.get_webview_window("main") {
            performance::window_state(window.is_visible().ok(), window.is_minimized().ok());
        }
    }
    performance::view()
}
#[tauri::command]
fn start_performance_capture(
    app: tauri::AppHandle,
    state: tauri::State<ShellState>,
) -> Result<performance::View, String> {
    if state.stopping.load(Ordering::Relaxed) {
        return Err("程序正在退出。".into());
    }
    let root = data_root(&std::env::current_exe().map_err(|_| "无法确定数据目录。")?)?;
    let initial = state
        .input
        .lock()
        .map_err(|_| "输入状态不可用。")?
        .status
        .clone();
    let range = *state
        .diagnostic_range
        .lock()
        .map_err(|_| "参数状态不可用。")?;
    let view =
        performance::start_with_window(root, initial, range, app.get_webview_window("main"))?;
    Ok(view)
}
#[tauri::command]
fn stop_performance_capture() -> performance::View {
    performance::stop()
}

fn data_root(exe: &Path) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    if let Some(root) = std::env::var_os("CLICKMASTER_DEV_ROOT") {
        return Ok(PathBuf::from(root));
    }
    exe.parent()
        .map(|parent| parent.join("data"))
        .ok_or_else(|| "无法确定程序目录。".into())
}

fn prepare_data(root: &Path) -> Result<(), String> {
    for name in ["config", "webview"] {
        fs::create_dir_all(root.join(name))
            .map_err(|_| "无法创建程序同级数据目录，请将程序移到可写目录后重试。")?;
        let probe = root
            .join(name)
            .join(format!(".write-check-{}", std::process::id()));
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe)
            .map_err(|_| "数据目录不可写，请检查权限或将程序移到可写目录后重试。")?;
        drop(file);
        fs::remove_file(probe).map_err(|_| "无法清理数据目录写入检查文件，请检查目录权限。")?;
    }
    Ok(())
}

fn restore(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let result = (|| -> tauri::Result<()> {
            window.set_skip_taskbar(false)?;
            window.show()?;
            window.unminimize()?;
            window.set_focus()?;
            Ok(())
        })();
        performance::window_state(window.is_visible().ok(), window.is_minimized().ok());
        if result.is_err() {
            native_error("无法还原主窗口，请检查桌面环境。程序仍可通过托盘退出。");
        }
    }
}

pub fn run() {
    if std::env::args_os().any(|arg| arg == "--ipc-worker") {
        if std::env::args_os().count() != 2 {
            std::process::exit(64);
        }
        std::process::exit(ipc_worker::entry());
    }
    if let Err(message) = run_shell() {
        native_error(&message);
    }
}

fn run_shell() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|_| "无法确定程序位置。")?;
    let root = data_root(&exe)?;
    prepare_data(&root)?;
    let state = ShellState {
        diagnostic_range: Mutex::new(None),
        tray_revision: AtomicU64::new(0),
        stop_requested: AtomicBool::new(false),
        hotkey: Mutex::new(None),
        tray_stop: Mutex::new(None),
        execution_gate: Mutex::new(()),
        channel: Mutex::new(channels::View::default()),
        ipc_worker: Mutex::new(None),
        store: Mutex::new(SettingsStore::load(&root)),
        tray_available: AtomicBool::new(false),
        close_pending: AtomicBool::new(false),
        stopping: AtomicBool::new(false),
        targets: Mutex::new(targets::TargetState::default()),
        overlay: Mutex::new(None),
        input: Mutex::new(input::Engine::default()),
        input_worker: Mutex::new(None),
        input_origin: std::time::Instant::now(),
    };
    let app = tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_performance_status,
            start_performance_capture,
            stop_performance_capture,
            get_app_info,
            get_channel_status,
            select_channel,
            set_minimize_preference,
            refresh_targets,
            bind_target,
            choose_receiver,
            unbind_target,
            get_binding_status,
            resolve_close,
            set_overlay,
            get_overlay_status,
            start_single_input,
            start_repeat_input,
            cancel_single_input,
            get_input_status
        ])
        .setup(move |app| {
            let window =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                    .title("ClickMaster")
                    .inner_size(800.0, 520.0)
                    .min_inner_size(520.0, 400.0)
                    .data_directory(root.join("webview"))
                    .visible(false)
                    .build()?;
            let open = MenuItem::with_id(app, "open", "打开主窗口", true, None::<&str>)?;
            let stop = MenuItem::with_id(app, "stop", "停止", false, None::<&str>)?;
            let exit = MenuItem::with_id(app, "exit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &stop, &exit])?;
            *app.state::<ShellState>()
                .tray_stop
                .lock()
                .map_err(|_| "tray state unavailable")? = Some(stop);
            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/icon.png"
                ))?)
                .tooltip("ClickMaster · 未运行")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => restore(app),
                    "exit" => app.exit(0),
                    "stop" => {
                        app.state::<ShellState>()
                            .stop_requested
                            .store(true, Ordering::Release);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        restore(tray.app_handle());
                    }
                })
                .build(app);
            app.state::<ShellState>()
                .tray_available
                .store(tray.is_ok(), Ordering::Relaxed);
            window.show()?;
            match input_runtime::Runtime::start(app.handle().clone()) {
                Ok(runtime) => {
                    *app.state::<ShellState>()
                        .input_worker
                        .lock()
                        .map_err(|_| "input state unavailable")? = Some(runtime);
                }
                Err(_) => {
                    let _ = app.emit(
                        "shell-warning",
                        "单次输入服务无法启动；不会发送输入，其他功能仍可使用。",
                    );
                }
            }
            match overlay_runtime::Runtime::start(app.handle().clone()) {
                Ok(runtime) => {
                    *app.state::<ShellState>()
                        .overlay
                        .lock()
                        .map_err(|_| "preview state unavailable")? = Some(runtime);
                }
                Err(_) => {
                    let _ = app.emit("shell-warning", "坐标预览服务无法启动；其他功能仍可使用。");
                }
            }
            // Independent of WebView timers, active page, and minimized/hidden state.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                while !handle
                    .state::<ShellState>()
                    .stopping
                    .load(Ordering::Relaxed)
                {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    let state = handle.state::<ShellState>();
                    if state.stopping.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Ok(mut targets) = state.targets.lock() {
                        let changed = targets.validate(&windows_query::NativeQuery);
                        let route_changed =
                            targets.validate_routing(&routing_native::NativeRouting);
                        if changed || route_changed {
                            let _ = handle.emit("binding-changed", targets.status());
                        }
                    };
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if performance::active() {
                performance::window_state(window.is_visible().ok(), window.is_minimized().ok());
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let state = window.state::<ShellState>();
                if !state.close_pending.swap(true, Ordering::Relaxed) {
                    if window.emit("close-choice-requested", ()).is_err() {
                        state.close_pending.store(false, Ordering::Relaxed);
                    }
                }
                return;
            }
            if matches!(event, WindowEvent::Resized(_)) && window.is_minimized().unwrap_or(false) {
                let state = window.state::<ShellState>();
                let keep = state
                    .store
                    .lock()
                    .map(|store| store.settings.minimize_keep_taskbar)
                    .unwrap_or(true);
                if !keep && state.tray_available.load(Ordering::Relaxed) {
                    if let Some(webview) = window.app_handle().get_webview_window("main") {
                        if let Err(message) = window_policy::tuck(&webview, false, true) {
                            let _ = window.emit("shell-warning", message);
                        }
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .map_err(|_| "ClickMaster 初始化失败，请确认已安装兼容的 WebView2，并检查数据目录权限。")?;
    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            let state = handle.state::<ShellState>();
            state.stopping.store(true, Ordering::Relaxed);
            let _ = cancel_input(&state, handle);
            let worker = state
                .input_worker
                .lock()
                .ok()
                .and_then(|mut worker| worker.take());
            if let Some(mut worker) = worker {
                worker.shutdown();
            }
            if let Ok(mut runtime) = handle.state::<ShellState>().overlay.lock() {
                if let Some(runtime) = runtime.as_mut() {
                    runtime.shutdown();
                }
            }
            // Input cancellation/owned-worker shutdown always precedes diagnostic IO grace.
            performance::shutdown();
            handle
                .state::<ShellState>()
                .stopping
                .store(true, Ordering::Relaxed);
        }
    });
    Ok(())
}

fn native_error(message: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let text: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    let title: Vec<u16> = "ClickMaster".encode_utf16().chain(Some(0)).collect();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        let root = PathBuf::from(
            std::env::var_os("CLICKMASTER_TEST_ROOT").expect("project test runner required"),
        )
        .join("data-root")
        .join(name);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn portable_unicode_space_move_upgrade_keeps_config_without_cwd_dependency() {
        let old = fixture("便携 测试/原 目录");
        let exe = old.join("ClickMaster.exe");
        std::fs::write(&exe, b"fixture executable marker").unwrap();
        let data = data_root(&exe).unwrap();
        prepare_data(&data).unwrap();
        let mut store = settings::SettingsStore::load(&data);
        store.save_preference(true).unwrap();
        let expected = std::fs::read(data.join("config/settings.json")).unwrap();
        drop(store);
        let moved = old.with_file_name("已移动 目录");
        std::fs::rename(&old, &moved).unwrap();
        std::fs::write(moved.join("ClickMaster.exe"), b"fixture upgrade marker").unwrap();
        let data = data_root(&moved.join("ClickMaster.exe")).unwrap();
        prepare_data(&data).unwrap();
        assert_eq!(
            std::fs::read(data.join("config/settings.json")).unwrap(),
            expected
        );
        assert!(
            settings::SettingsStore::load(&data)
                .settings
                .minimize_keep_taskbar
        );
    }
    #[test]
    fn stop_service_error_never_overwrites_partial_or_cleanup_lock() {
        for phase in ["partial", "cleanup-failed", "timeout"] {
            let mut status = input::Status::default();
            status.phase = phase.into();
            status.busy = phase == "cleanup-failed";
            stop_service_failure(&mut status);
            assert_eq!(status.phase, phase);
            assert_eq!(status.busy, phase == "cleanup-failed");
        }
        let mut status = input::Status::default();
        status.phase = "cancelled".into();
        stop_service_failure(&mut status);
        assert_eq!(status.phase, "failed");
        assert_eq!(status.timing.end_reason.as_deref(), Some("停止服务异常"));
    }
    #[test]
    fn production_root_follows_executable_instead_of_working_directory() {
        let exe = fixture("location").join("portable/ClickMaster.exe");
        assert_eq!(data_root(&exe).unwrap(), exe.parent().unwrap().join("data"));
    }

    #[test]
    fn blocked_config_directory_fails_before_webview_directory_creation() {
        let root = fixture("blocked");
        fs::write(root.join("config"), b"blocked directory").unwrap();
        assert!(prepare_data(&root).is_err());
        assert!(!root.join("webview").exists());
    }

    #[test]
    fn writable_root_has_no_remaining_probe_files() {
        let root = fixture("writable");
        prepare_data(&root).unwrap();
        for name in ["config", "webview"] {
            assert_eq!(fs::read_dir(root.join(name)).unwrap().count(), 0);
        }
    }
}
