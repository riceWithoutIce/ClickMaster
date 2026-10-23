//! UI-independent supervisor. Only a directly owned Child may be terminated.
use crate::{
    input,
    ipc::{self, Report},
    ipc_native as native, ipc_worker, ShellState,
};
use std::{
    io::Write,
    os::windows::process::CommandExt,
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tauri::Manager;

pub struct Runtime {
    cancel: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}
fn update(app: &tauri::AppHandle, phase: &str, receipt: ipc::Receipt, remaining: u64, busy: bool) {
    if let Ok(mut engine) = app.state::<ShellState>().input.lock() {
        let s = &mut engine.status;
        if s.phase == phase
            && s.ipc.as_ref() == Some(&receipt)
            && s.remaining_ms.div_ceil(100) == remaining.div_ceil(100)
            && s.busy == busy
        {
            return;
        }
        s.revision += 1;
        s.phase = phase.into();
        s.message = if let Some(failure) = receipt.failure.as_ref().filter(|_| {
            !matches!(
                phase,
                "cancelled" | "expired" | "stopping" | "cleanup-failed"
            )
        }) {
            failure.message()
        } else if phase == "target-invalid" {
            "原绑定目标失效，执行已停止。"
        } else if phase == "expired" {
            "时长已到，已停止并清理；已发帧无法撤回。"
        } else {
            ipc::phase_message(phase)
        }
        .into();
        if !busy || phase == "cleanup-failed" {
            s.timing.finish(
                crate::input_runtime::elapsed(app.state::<ShellState>().input_origin),
                crate::duration::reason(phase),
            );
            s.timing.end_reason = Some(crate::duration::reason(phase).into());
        }
        s.ipc = Some(receipt);
        s.remaining_ms = remaining;
        s.busy = busy;
        crate::publish_input(app, s);
    };
}
fn timing(app: &tauri::AppHandle, begin: bool, finish: Option<&str>) -> Result<bool, String> {
    let state = app.state::<ShellState>();
    let now = crate::input_runtime::elapsed(state.input_origin);
    let mut engine = state.input.lock().map_err(|_| "执行时钟不可用。")?;
    let before = engine.status.timing.clone();
    let clock = &mut engine.status.timing;
    if begin {
        clock.begin(now)?;
    }
    clock.refresh(now);
    let expired = clock.expired(now);
    if let Some(phase) = finish {
        clock.finish(now, crate::duration::reason(phase));
    }
    if engine.status.timing != before {
        engine.status.revision += 1;
    }
    Ok(expired)
}
fn interrupted(app: &tauri::AppHandle, stop: &AtomicBool, phase: &str) -> Option<&'static str> {
    crate::duration::interruption(
        phase,
        stop.load(Ordering::Acquire),
        timing(app, false, None).unwrap_or(true),
    )
}
pub fn cleanup(child: &mut Child) -> bool {
    let _measure = crate::performance::span(crate::performance::Stage::Cleanup);
    if matches!(child.try_wait(), Ok(Some(_))) {
        return true;
    }
    // std::process::Child retains the actual owned process handle, avoiding PID reuse.
    let _ = child.kill();
    let deadline = Instant::now();
    while deadline.elapsed() < Duration::from_secs(2) {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

struct Observation {
    phase: String,
    receipt: ipc::Receipt,
    worker_elapsed: u64,
    stage: Instant,
    written_at: Option<Instant>,
    sequence: u64,
}
impl Observation {
    fn new(origin: Instant) -> Self {
        Self {
            phase: "checking".into(),
            receipt: ipc::Receipt::default(),
            worker_elapsed: 0,
            stage: origin,
            written_at: None,
            sequence: 0,
        }
    }
    fn refresh(&mut self, dir: &Path, origin: Instant) -> Result<(), ()> {
        let Ok(bytes) = std::fs::read(dir.join("status.json")) else {
            return Ok(());
        };
        if bytes.len() > 4096 {
            return Err(());
        }
        let Ok(report) = serde_json::from_slice::<Report>(&bytes) else {
            return Err(());
        };
        if report.performance.as_ref().is_some_and(|m| !m.valid()) {
            return Err(());
        }
        if report.elapsed_ms < self.worker_elapsed
            || report.sequence < self.sequence
            || report.elapsed_ms > ipc::TOTAL_MS
            || report.stage_ms > report.elapsed_ms
            || (self.receipt.written && !report.receipt.written)
            || (report.receipt.written && !report.receipt.connected)
        {
            return Err(());
        }
        self.worker_elapsed = report.elapsed_ms;
        self.sequence = report.sequence;
        if self.phase != report.phase {
            self.phase = report.phase;
            self.stage = origin + Duration::from_millis(report.elapsed_ms - report.stage_ms);
        }
        if report.receipt.written && self.written_at.is_none() {
            self.written_at = Some(Instant::now());
        }
        let first_failure = self.receipt.failure.take();
        self.receipt = report.receipt;
        if let Some(failure) = first_failure {
            self.receipt.failure = Some(failure);
        }
        Ok(())
    }
}

struct PersistentSession {
    child: Child,
    input: ChildStdin,
    dir: std::path::PathBuf,
    sequence: u64,
}

impl PersistentSession {
    fn abort(child: &mut Child, failure: ipc::Failure) -> ipc::Failure {
        if cleanup(child) {
            crate::performance::finish_child(child);
            failure
        } else {
            ipc::Failure::new(
                ipc::FailureStage::Cleanup,
                ipc::FailureReason::CleanupUnconfirmed,
            )
        }
    }
    fn start(
        app: &tauri::AppHandle,
        lease: &native::Lease,
        stop: &AtomicBool,
        slot: &mut crate::ipc_storage::Slot,
    ) -> Result<Self, ipc::Failure> {
        let setup_failure =
            || ipc::Failure::new(ipc::FailureStage::Worker, ipc::FailureReason::SetupFailed);
        let exe = std::env::current_exe().map_err(|_| setup_failure())?;
        let (dir, token) = slot.prepare().map_err(|_| {
            ipc::Failure::new(
                ipc::FailureStage::Authorization,
                ipc::FailureReason::SetupFailed,
            )
        })?;
        let (_, parent_created, _) = native::process(std::process::id())?;
        let auth = ipc_worker::Authorization {
            token,
            target: lease.identity.clone(),
            parent: std::process::id(),
            parent_created,
            diagnostics: crate::performance::epoch(),
        };
        let bytes = serde_json::to_vec(&auth).map_err(|_| {
            ipc::Failure::new(
                ipc::FailureStage::Authorization,
                ipc::FailureReason::AuthorizationFailed,
            )
        })?;
        let length = u32::try_from(bytes.len()).map_err(|_| {
            ipc::Failure::new(
                ipc::FailureStage::Authorization,
                ipc::FailureReason::AuthorizationFailed,
            )
        })?;
        let temp = dir.join("tmp");
        let mut child = {
            let _measure = crate::performance::span(crate::performance::Stage::Spawn);
            Command::new(&exe)
                .arg("--ipc-worker")
                .env(
                    "CLICKMASTER_IPC_RUN",
                    dir.file_name().ok_or_else(setup_failure)?,
                )
                .env("CLICKMASTER_IPC_PERSISTENT", "1")
                .env("TEMP", &temp)
                .env("TMP", &temp)
                .env("TMPDIR", &temp)
                .current_dir(exe.parent().ok_or_else(setup_failure)?)
                .creation_flags(0x08000000)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|error| {
                    let mut failure = setup_failure();
                    failure.error_code = error
                        .raw_os_error()
                        .and_then(|code| u32::try_from(code).ok());
                    failure
                })?
        };
        crate::performance::observe_child(&child, true);
        let mut input = child.stdin.take().ok_or_else(|| {
            Self::abort(
                &mut child,
                ipc::Failure::new(
                    ipc::FailureStage::Authorization,
                    ipc::FailureReason::AuthorizationFailed,
                ),
            )
        })?;
        if input.write_all(&length.to_le_bytes()).is_err()
            || input.write_all(&bytes).is_err()
            || input.flush().is_err()
        {
            return Err(Self::abort(
                &mut child,
                ipc::Failure::new(
                    ipc::FailureStage::Authorization,
                    ipc::FailureReason::AuthorizationFailed,
                ),
            ));
        }
        let mut last = ipc::Receipt::default();
        let deadline = Instant::now() + Duration::from_millis(ipc::CONNECT_MS);
        loop {
            crate::performance::observe_child(&child, false);
            if let Err(failure) = lease.validate_fast() {
                return Err(Self::abort(&mut child, failure));
            }
            if let Some(reason) = interrupted(app, stop, "connecting") {
                return Err(Self::abort(
                    &mut child,
                    ipc::Failure::new(
                        ipc::FailureStage::Supervision,
                        if reason == "expired" {
                            ipc::FailureReason::Timeout
                        } else {
                            ipc::FailureReason::AuthorizationFailed
                        },
                    ),
                ));
            }
            if let Ok(bytes) = std::fs::read(dir.join("status.json")) {
                if bytes.len() > 4096 {
                    return Err(Self::abort(
                        &mut child,
                        ipc::Failure::new(
                            ipc::FailureStage::Status,
                            ipc::FailureReason::ReportInvalid,
                        ),
                    ));
                }
                if let Ok(report) = serde_json::from_slice::<Report>(&bytes) {
                    if report.performance.as_ref().is_some_and(|m| !m.valid()) {
                        return Err(Self::abort(
                            &mut child,
                            ipc::Failure::new(
                                ipc::FailureStage::Status,
                                ipc::FailureReason::ReportInvalid,
                            ),
                        ));
                    }
                    last = report.receipt.clone();
                    if report.phase == "connected" && last.connected {
                        return Ok(Self {
                            child,
                            input,
                            dir,
                            sequence: report.sequence,
                        });
                    }
                    update(app, &report.phase, report.receipt, 0, true);
                    if report.phase == "failed" {
                        return Err(Self::abort(
                            &mut child,
                            last.failure.unwrap_or_else(|| {
                                ipc::Failure::new(
                                    ipc::FailureStage::Connect,
                                    ipc::FailureReason::SystemCall,
                                )
                            }),
                        ));
                    }
                }
            }
            if matches!(child.try_wait(), Ok(Some(_))) {
                return Err(last.failure.unwrap_or_else(|| {
                    ipc::Failure::new(ipc::FailureStage::Worker, ipc::FailureReason::WorkerExited)
                }));
            }
            if Instant::now() >= deadline {
                return Err(Self::abort(
                    &mut child,
                    ipc::Failure::new(ipc::FailureStage::Connect, ipc::FailureReason::Timeout),
                ));
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    fn send(
        &mut self,
        app: &tauri::AppHandle,
        lease: &native::Lease,
        stop: &AtomicBool,
    ) -> (&'static str, ipc::Receipt, bool) {
        // The connection may have taken seconds. Recheck cancellation at the
        // command admission boundary before allowing the worker's next write.
        if let Some(reason) = interrupted(app, stop, "checking") {
            return (reason, ipc::Receipt::default(), true);
        }
        if let Err(failure) = lease.validate_fast() {
            return (
                "target-invalid",
                ipc::Receipt {
                    failure: Some(failure),
                    ..Default::default()
                },
                true,
            );
        }
        if self
            .input
            .write_all(&[1])
            .and_then(|_| self.input.flush())
            .is_err()
        {
            let mut receipt = ipc::Receipt::default();
            receipt.fail(ipc::Failure::new(
                ipc::FailureStage::Worker,
                ipc::FailureReason::WorkerExited,
            ));
            return ("failed", receipt, true);
        }
        self.wait(
            || {
                interrupted(app, stop, "writing")
                    .map(|reason| (reason, None))
                    .or_else(|| {
                        lease
                            .validate_fast()
                            .err()
                            .map(|failure| ("target-invalid", Some(failure)))
                    })
            },
            |phase, receipt| update(app, phase, receipt.clone(), 0, true),
        )
    }

    /// The same bounded sequence observer is exercised by the non-game fixtures.
    fn wait(
        &mut self,
        mut control: impl FnMut() -> Option<(&'static str, Option<ipc::Failure>)>,
        mut observe: impl FnMut(&str, &ipc::Receipt),
    ) -> (&'static str, ipc::Receipt, bool) {
        let target = self.sequence.saturating_add(1);
        let began = Instant::now();
        let deadline = Instant::now() + Duration::from_millis(ipc::WRITE_MS + 2_000);
        let mut receipt = ipc::Receipt::default();
        let outcome = loop {
            crate::performance::observe_child(&self.child, false);
            if let Some((reason, failure)) = control() {
                if let Some(failure) = failure {
                    receipt.fail(failure);
                }
                break reason;
            }
            if let Ok(bytes) = std::fs::read(self.dir.join("status.json")) {
                if bytes.len() > 4096 {
                    receipt.fail(ipc::Failure::new(
                        ipc::FailureStage::Status,
                        ipc::FailureReason::ReportInvalid,
                    ));
                    break "failed";
                }
                if let Ok(report) = serde_json::from_slice::<Report>(&bytes) {
                    if report.sequence > target
                        || (report.receipt.written && !report.receipt.connected)
                    {
                        receipt.fail(ipc::Failure::new(
                            ipc::FailureStage::Status,
                            ipc::FailureReason::ReportInvalid,
                        ));
                        break "failed";
                    }
                    if report.sequence == target {
                        receipt = report.receipt.clone();
                        // Short frames do not emit two UI events each. A slow or
                        // blocked frame still surfaces its phase; the existing
                        // backend clock publishes the running count periodically.
                        if began.elapsed() >= Duration::from_millis(100) {
                            observe(&report.phase, &report.receipt);
                        }
                        if report.sequence >= target && report.phase == "completed" {
                            self.sequence = report.sequence;
                            receipt.finish_observation();
                            break "completed";
                        }
                        if report.sequence >= target && report.phase == "failed" {
                            self.sequence = report.sequence;
                            break "failed";
                        }
                    }
                }
            }
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                receipt.fail(ipc::Failure::new(
                    ipc::FailureStage::Worker,
                    ipc::FailureReason::WorkerExited,
                ));
                break "failed";
            }
            if Instant::now() >= deadline {
                receipt.fail(ipc::Failure::new(
                    ipc::FailureStage::Write,
                    ipc::FailureReason::Timeout,
                ));
                break "timeout";
            }
            thread::sleep(Duration::from_millis(1));
        };
        let clean = outcome == "completed" || cleanup(&mut self.child);
        if outcome != "completed" && clean {
            let first_failure = receipt.failure.take();
            if let Ok(bytes) = std::fs::read(self.dir.join("status.json")) {
                if bytes.len() <= 4096 {
                    if let Ok(report) = serde_json::from_slice::<Report>(&bytes) {
                        if report.sequence == target {
                            receipt = report.receipt;
                        }
                    }
                }
            }
            if let Some(first) = first_failure {
                receipt.failure = Some(first);
            }
        }
        receipt.finish_observation();
        (outcome, receipt, clean)
    }

    fn shutdown(&mut self) -> bool {
        let _ = self.input.flush();
        let clean = cleanup(&mut self.child);
        crate::performance::finish_child(&self.child);
        if crate::performance::active() {
            let report = std::fs::read(self.dir.join("status.json"))
                .ok()
                .filter(|b| b.len() <= 4096)
                .and_then(|b| serde_json::from_slice::<Report>(&b).ok());
            crate::performance::merge_worker(
                report.as_ref().and_then(|r| r.performance.as_ref()),
                report.as_ref().is_none_or(|r| r.phase != "completed"),
                report.as_ref().map_or(0, |r| r.performance_epoch),
            );
        }
        clean
    }
}
/// Shared with offline pipe fixtures: no AppHandle, game lookup, hotkey or desktop input.
fn supervise(
    child: &mut Child,
    dir: &Path,
    origin: Instant,
    mut control: impl FnMut(&str, &ipc::Receipt) -> Option<&'static str>,
    mut observe: impl FnMut(&str, &ipc::Receipt),
) -> (&'static str, ipc::Receipt, bool) {
    let mut state = Observation::new(origin);
    let mut outcome = loop {
        crate::performance::observe_child(child, false);
        if state.refresh(dir, origin).is_err() {
            state.receipt.fail(ipc::Failure::new(
                ipc::FailureStage::Status,
                ipc::FailureReason::ReportInvalid,
            ));
            break "failed";
        }
        observe(&state.phase, &state.receipt);
        // Manual/deadline/identity protection remains independent of read diagnostics.
        if let Some(reason) = control(&state.phase, &state.receipt) {
            break reason;
        }
        match child.try_wait() {
            Ok(Some(_)) => {
                // The worker may have published its final receipt after the earlier read.
                // Re-read only after observing exit, when it can no longer update the file.
                if state.refresh(dir, origin).is_err() {
                    state.receipt.fail(ipc::Failure::new(
                        ipc::FailureStage::Status,
                        ipc::FailureReason::ReportInvalid,
                    ));
                    break "failed";
                }
                break if state.receipt.failure.is_some() {
                    if state.phase == "timeout" {
                        "timeout"
                    } else {
                        "failed"
                    }
                } else if state.receipt.written {
                    "completed"
                } else if state.phase == "timeout" {
                    "timeout"
                } else {
                    "failed"
                };
            }
            Err(error) => {
                let mut failure = ipc::Failure::new(
                    ipc::FailureStage::Supervision,
                    ipc::FailureReason::SystemCall,
                );
                failure.error_code = error
                    .raw_os_error()
                    .and_then(|code| u32::try_from(code).ok());
                state.receipt.fail(failure);
                break "failed";
            }
            _ => {}
        }
        if state.receipt.written {
            if state
                .written_at
                .is_some_and(|at| at.elapsed() >= Duration::from_millis(ipc::READ_OBSERVE_MS))
            {
                // A read failure or missing confirmation never blocks the next send.
                break "completed";
            }
        } else if ipc::expired(
            origin.elapsed().as_millis() as u64,
            state.stage.elapsed().as_millis() as u64,
            0,
            &state.phase,
        ) {
            break "timeout";
        }
        thread::sleep(Duration::from_millis(10));
    };
    let clean = cleanup(child);
    crate::performance::finish_child(child);
    if clean {
        // Also collect a complete write that raced with stop/deadline, before slot reuse.
        // Preserve the stop/error outcome; only the receipt and sent count are updated.
        let _ = state.refresh(dir, origin);
    }
    if outcome == "completed" && state.receipt.failure.is_some() {
        outcome = "failed";
    }
    if outcome == "failed" && state.receipt.failure.is_none() {
        state.receipt.fail(ipc::Failure::new(
            ipc::FailureStage::Worker,
            ipc::FailureReason::WorkerExited,
        ));
    } else if outcome == "timeout" {
        state.receipt.fail(ipc::Failure::new(
            ipc::FailureStage::Supervision,
            ipc::FailureReason::Timeout,
        ));
    }
    if !clean {
        state.receipt.fail(ipc::Failure::new(
            ipc::FailureStage::Cleanup,
            ipc::FailureReason::CleanupUnconfirmed,
        ));
    }
    state.receipt.finish_observation();
    if crate::performance::active() {
        let report = std::fs::read(dir.join("status.json"))
            .ok()
            .filter(|b| b.len() <= 4096)
            .and_then(|b| serde_json::from_slice::<Report>(&b).ok());
        crate::performance::merge_worker(
            report.as_ref().and_then(|r| r.performance.as_ref()),
            report.as_ref().is_none_or(|r| r.phase != "completed"),
            report.as_ref().map_or(0, |r| r.performance_epoch),
        );
    }
    (outcome, state.receipt, clean)
}

fn transfer(
    app: &tauri::AppHandle,
    lease: &native::Lease,
    stop: &AtomicBool,
    slot: &mut crate::ipc_storage::Slot,
) -> (&'static str, ipc::Receipt, bool) {
    let mut receipt = ipc::Receipt::default();
    if let Some(reason) = interrupted(app, stop, "checking") {
        return (reason, receipt, true);
    }
    let origin = Instant::now();
    let setup = (|| -> Result<_, ipc::Failure> {
        let setup_failure =
            || ipc::Failure::new(ipc::FailureStage::Worker, ipc::FailureReason::SetupFailed);
        let exe = std::env::current_exe().map_err(|_| setup_failure())?;
        let (dir, token) = slot.prepare().map_err(|_| {
            ipc::Failure::new(
                ipc::FailureStage::Authorization,
                ipc::FailureReason::SetupFailed,
            )
        })?;
        let (_, parent_created, _) = native::process(std::process::id())?;
        let auth = ipc_worker::Authorization {
            token,
            target: lease.identity.clone(),
            parent: std::process::id(),
            parent_created,
            diagnostics: crate::performance::epoch(),
        };
        let temp = dir.join("tmp");
        let _measure = crate::performance::span(crate::performance::Stage::Spawn);
        let child = Command::new(&exe)
            .arg("--ipc-worker")
            .env(
                "CLICKMASTER_IPC_RUN",
                dir.file_name().ok_or_else(setup_failure)?,
            )
            .env("TEMP", &temp)
            .env("TMP", &temp)
            .env("TMPDIR", &temp)
            .current_dir(exe.parent().ok_or_else(setup_failure)?)
            .creation_flags(0x08000000)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                let mut failure = setup_failure();
                failure.error_code = error
                    .raw_os_error()
                    .and_then(|code| u32::try_from(code).ok());
                failure
            })?;
        crate::performance::observe_child(&child, true);
        Ok((dir, auth, child))
    })();
    let (dir, auth, mut child) = match setup {
        Ok(value) => value,
        Err(failure) => {
            receipt.fail(failure);
            return ("failed", receipt, true);
        }
    };
    // Worker cannot proceed without stdin authorization: recheck immediately before releasing it.
    if let Some(reason) = interrupted(app, stop, "checking") {
        let _ = timing(app, false, Some(reason));
        return (reason, receipt, cleanup(&mut child));
    }
    let auth_ok = child.stdin.take().is_some_and(|mut input| {
        serde_json::to_vec(&auth)
            .ok()
            .is_some_and(|bytes| input.write_all(&bytes).is_ok())
    });
    if !auth_ok {
        receipt.fail(ipc::Failure::new(
            ipc::FailureStage::Authorization,
            ipc::FailureReason::AuthorizationFailed,
        ));
        return ("failed", receipt, cleanup(&mut child));
    }
    let mut target_failure = None;
    let (outcome, mut receipt, clean) = supervise(
        &mut child,
        &dir,
        origin,
        |phase, observed| {
            interrupted(app, stop, phase).or_else(|| {
                lease.validate_fast().err().map(|error| {
                    if observed.failure.is_none() {
                        target_failure = Some(error);
                    }
                    "target-invalid"
                })
            })
        },
        |phase, receipt| update(app, phase, receipt.clone(), 0, true),
    );
    if let Some(failure) = target_failure {
        // Captured before cleanup: a later worker report must not replace this first cause.
        receipt.failure = Some(failure);
    }
    if outcome != "completed" {
        let _ = timing(app, false, Some(outcome));
    }
    if matches!(outcome, "expired" | "cancelled") {
        update(app, "stopping", receipt.clone(), 0, true);
    }
    (outcome, receipt, clean)
}
impl Runtime {
    pub fn start(
        app: tauri::AppHandle,
        lease: native::Lease,
        range: Option<crate::repeat::Range>,
    ) -> Result<Self, String> {
        let cancel = Arc::new(AtomicBool::new(false));
        let stop = cancel.clone();
        let join = thread::Builder::new()
            .name("ipc-supervisor".into())
            .spawn(move || {
                let root = std::env::current_exe()
                    .ok()
                    .and_then(|exe| crate::data_root(&exe).ok());
                let Some(root) = root else {
                    let mut receipt = ipc::Receipt::default();
                    receipt.fail(ipc::Failure::new(
                        ipc::FailureStage::Worker,
                        ipc::FailureReason::SetupFailed,
                    ));
                    update(&app, "failed", receipt, 0, false);
                    return;
                };
                let mut slot = crate::ipc_storage::Slot::new(&root);
                // Repeating IPC uses one bounded worker and pipe for the run;
                // single execution keeps the established one-frame path.
                let persistent_mode = range.is_some();
                let mut persistent: Option<PersistentSession> = None;
                let mut setup_cleanup_confirmed = true;
                let mut receipt = ipc::Receipt::default();
                let clock = Instant::now();
                let mut due = input::COUNTDOWN_MS;
                let mut phase = "countdown";
                let outcome = loop {
                    while (clock.elapsed().as_millis() as u64) < due {
                        if interrupted(&app, &stop, phase).is_some() {
                            break;
                        }
                        if let Err(failure) = lease.validate_fast() {
                            receipt.fail(failure);
                            break;
                        }
                        update(
                            &app,
                            phase,
                            receipt.clone(),
                            due.saturating_sub(clock.elapsed().as_millis() as u64),
                            true,
                        );
                        let remaining = due.saturating_sub(clock.elapsed().as_millis() as u64);
                        thread::sleep(Duration::from_millis(remaining.clamp(1, 50)));
                    }
                    if let Some(reason) = interrupted(&app, &stop, phase) {
                        break reason;
                    }
                    if receipt.failure.is_some() {
                        break "target-invalid";
                    }
                    if let Err(failure) = lease.validate_fast() {
                        receipt.fail(failure);
                        break "target-invalid";
                    }
                    if app
                        .state::<ShellState>()
                        .input
                        .lock()
                        .map(|engine| engine.status.completed >= crate::repeat::MAX_COMPLETED)
                        .unwrap_or(true)
                    {
                        receipt.fail(ipc::Failure::new(
                            ipc::FailureStage::Supervision,
                            ipc::FailureReason::CounterLimit,
                        ));
                        break "failed";
                    }
                    if timing(&app, true, None).is_err() {
                        receipt.fail(ipc::Failure::new(
                            ipc::FailureStage::Supervision,
                            ipc::FailureReason::SystemCall,
                        ));
                        break "failed";
                    }
                    if let Some(reason) = interrupted(&app, &stop, "checking") {
                        break reason;
                    }
                    if persistent.is_none() {
                        update(&app, "checking", receipt.clone(), 0, true);
                    }
                    let (outcome, new_receipt, clean) = if persistent_mode {
                        if persistent.is_none() {
                            match PersistentSession::start(&app, &lease, &stop, &mut slot) {
                                Ok(value) => persistent = Some(value),
                                Err(failure) => {
                                    if failure.reason == ipc::FailureReason::CleanupUnconfirmed {
                                        setup_cleanup_confirmed = false;
                                    }
                                    if let Some(reason) = interrupted(&app, &stop, "connecting") {
                                        break reason;
                                    }
                                    receipt.fail(failure);
                                    break "failed";
                                }
                            }
                        }
                        persistent
                            .as_mut()
                            .expect("persistent session initialized")
                            .send(&app, &lease, &stop)
                    } else {
                        transfer(&app, &lease, &stop, &mut slot)
                    };
                    receipt = new_receipt;
                    // One transfer has at most one complete write. Count even if it ended
                    // by stop/deadline or cleanup failure after that write.
                    let counted = app
                        .state::<ShellState>()
                        .input
                        .lock()
                        .map(|mut engine| ipc::record_sent(&mut engine.status.completed, &receipt))
                        .unwrap_or(false);
                    if !clean {
                        receipt.fail(ipc::Failure::new(
                            ipc::FailureStage::Cleanup,
                            ipc::FailureReason::CleanupUnconfirmed,
                        ));
                        let _ = timing(&app, false, Some("cleanup-failed"));
                        update(&app, "cleanup-failed", receipt, 0, true);
                        return;
                    }
                    if !counted {
                        receipt.fail(ipc::Failure::new(
                            ipc::FailureStage::Supervision,
                            ipc::FailureReason::CounterLimit,
                        ));
                        break "failed";
                    }
                    if outcome != "completed" {
                        break outcome;
                    }
                    let Some(range) = range else {
                        break "completed";
                    };
                    if let Some(reason) = interrupted(&app, &stop, "completed") {
                        break reason;
                    }
                    let delay = range.sample(native::random_u32);
                    let Ok(delay) = delay else {
                        receipt.fail(ipc::Failure::new(
                            ipc::FailureStage::Supervision,
                            ipc::FailureReason::RandomUnavailable,
                        ));
                        break "failed";
                    };
                    let Ok(next) =
                        crate::repeat::next_due(clock.elapsed().as_millis() as u64, delay)
                    else {
                        receipt.fail(ipc::Failure::new(
                            ipc::FailureStage::Supervision,
                            ipc::FailureReason::CounterLimit,
                        ));
                        break "failed";
                    };
                    due = next;
                    phase = "waiting";
                };
                let _ = timing(&app, false, Some(outcome));
                if matches!(outcome, "expired" | "cancelled") {
                    update(&app, "stopping", receipt.clone(), 0, true);
                }
                let session_clean = persistent
                    .as_mut()
                    .map_or(true, PersistentSession::shutdown);
                let clean = setup_cleanup_confirmed && session_clean && slot.finish().is_ok();
                if !clean {
                    receipt.fail(ipc::Failure::new(
                        ipc::FailureStage::Cleanup,
                        ipc::FailureReason::CleanupUnconfirmed,
                    ));
                }
                update(
                    &app,
                    if clean { outcome } else { "cleanup-failed" },
                    receipt,
                    0,
                    !clean,
                );
            })
            .map_err(|_| "无法启动 IPC 监督服务。")?;
        Ok(Self {
            cancel,
            join: Some(join),
        })
    }
    pub fn shutdown(&mut self) {
        self.cancel.store(true, Ordering::Release);
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
    use std::{fs, io::Read, path::PathBuf};
    fn fixture_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(
            std::env::var_os("CLICKMASTER_TEST_ROOT").expect("project test runner required"),
        )
        .join("ipc-fixtures")
        .join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }
    fn child(mode: &str) -> Command {
        let mut cmd = Command::new(std::env::current_exe().unwrap());
        cmd.args([
            "--exact",
            "ipc_runtime::tests::fixture_child",
            "--ignored",
            "--test-threads=1",
        ])
        .env("CLICKMASTER_FIXTURE", mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x08000000);
        cmd
    }
    #[test]
    #[ignore = "subprocess fixture invoked only with private test environment"]
    fn fixture_child() {
        match std::env::var("CLICKMASTER_FIXTURE").unwrap().as_str() {
            "sleep" => thread::sleep(Duration::from_secs(60)),
            "read" => {
                let name = std::env::var("CLICKMASTER_FIXTURE_PIPE").unwrap();
                assert!(name.starts_with(r"\\.\pipe\ClickMaster-OfflineFixture-"));
                let mut pipe = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(name)
                    .unwrap();
                let mut bytes = [0; 4];
                pipe.read_exact(&mut bytes).unwrap();
                assert_eq!(bytes, ipc::FRAME);
                thread::sleep(Duration::from_millis(200));
            }
            "persistent-server" => {
                let name = std::env::var("CLICKMASTER_FIXTURE_PIPE").unwrap();
                assert!(name.starts_with(r"\\.\pipe\ClickMaster-OfflineFixture-"));
                let dir = PathBuf::from(std::env::var_os("CLICKMASTER_FIXTURE_DIR").unwrap());
                let pipe = native::pipe(&name).unwrap();
                fs::write(dir.join("ready"), "ready").unwrap();
                native::connect_bounded(&pipe, 2000).unwrap();
                let mut input = std::io::stdin().lock();
                let mut sequence = 0;
                while let Ok(()) = input.read_exact(&mut [0; 1]) {
                    sequence += 1;
                    let mut report = Report::new("writing");
                    report.sequence = sequence;
                    let result = ipc_worker::send_persistent_frame(
                        &pipe,
                        || Ok(()),
                        |phase, receipt| {
                            report.phase = phase.into();
                            report.receipt = receipt;
                            ipc_worker::report(&dir, &report)
                        },
                    );
                    if let Err(failure) = result {
                        report.phase = "failed".into();
                        report.receipt.fail(failure);
                        ipc_worker::report(&dir, &report).unwrap();
                        break;
                    }
                }
            }
            "persistent-pressure" => {
                let name = std::env::var("CLICKMASTER_FIXTURE_PIPE").unwrap();
                assert!(name.starts_with(r"\\.\pipe\ClickMaster-OfflineFixture-"));
                let dir = PathBuf::from(std::env::var_os("CLICKMASTER_FIXTURE_DIR").unwrap());
                let pipe = native::pipe(&name).unwrap();
                fs::write(dir.join("ready"), "ready").unwrap();
                native::connect_bounded(&pipe, 2000).unwrap();
                let start = Instant::now();
                let mut sent = 0;
                loop {
                    match native::write(&pipe) {
                        Ok(()) => sent += 1,
                        Err(failure) => {
                            assert_eq!(failure.reason, ipc::FailureReason::Timeout);
                            fs::write(dir.join("pressure.json"), serde_json::to_vec(&serde_json::json!({
                                "completeFramesWritten":sent,"elapsedMicros":start.elapsed().as_micros() as u64,
                                "failure":failure,"consumerReadFrames":0,"reconnections":0,
                            })).unwrap()).unwrap();
                            break;
                        }
                    }
                    assert!(sent < 100_000);
                }
            }
            "drain-server" => {
                let name = std::env::var("CLICKMASTER_FIXTURE_PIPE").unwrap();
                assert!(name.starts_with(r"\\.\pipe\ClickMaster-OfflineFixture-"));
                let dir = PathBuf::from(std::env::var_os("CLICKMASTER_FIXTURE_DIR").unwrap());
                let pipe = native::pipe(&name).unwrap();
                fs::write(dir.join("ready"), "ready").unwrap();
                native::connect_bounded(&pipe, 2000).unwrap();
                native::write(&pipe).unwrap();
                fs::write(dir.join("draining"), "written").unwrap();
                native::drain(&pipe).unwrap();
            }
            "observed-server" => {
                let name = std::env::var("CLICKMASTER_FIXTURE_PIPE").unwrap();
                assert!(name.starts_with(r"\\.\pipe\ClickMaster-OfflineFixture-"));
                let dir = PathBuf::from(std::env::var_os("CLICKMASTER_FIXTURE_DIR").unwrap());
                let behavior = std::env::var("CLICKMASTER_FIXTURE_BEHAVIOR").unwrap();
                let pipe = native::pipe(&name).unwrap();
                fs::write(dir.join("ready"), "ready").unwrap();
                native::connect_bounded(&pipe, 2000).unwrap();
                if behavior == "write-failure" {
                    fs::write(dir.join("connected"), "connected").unwrap();
                    await_marker(&dir.join("allow-write"));
                }
                let origin = Instant::now();
                let mut current = Report::new("checking");
                let mut stage = origin;
                let result = ipc_worker::send_frame_with_drain(
                    &pipe,
                    || Ok(()),
                    |phase, receipt| {
                        if current.phase != phase {
                            stage = Instant::now();
                        }
                        current.phase = phase.into();
                        current.receipt = receipt;
                        if phase == "completed" && behavior == "late-failure" {
                            current.receipt.fail(
                                ipc::Failure::new(
                                    ipc::FailureStage::Status,
                                    ipc::FailureReason::ReportInvalid,
                                )
                                .code(5),
                            );
                        }
                        current.elapsed_ms = origin.elapsed().as_millis() as u64;
                        current.stage_ms = stage.elapsed().as_millis() as u64;
                        if phase == "completed" && behavior == "final-race" {
                            await_marker(&dir.join("allow-final"));
                        }
                        ipc_worker::report(&dir, &current)?;
                        if phase == "writing" && behavior == "wait-write" {
                            await_marker(&dir.join("allow-write"));
                        }
                        if phase == "draining" && behavior == "read-failure" {
                            await_marker(&dir.join("allow-drain"));
                        }
                        if phase == "completed" && behavior == "final-race" {
                            fs::write(dir.join("final-published"), "done").unwrap();
                        }
                        Ok(())
                    },
                    |pipe| {
                        if behavior == "read-failure" {
                            Err(109)
                        } else {
                            native::drain(pipe)
                        }
                    },
                );
                if let Err(failure) = result {
                    current.phase = "failed".into();
                    current.receipt.fail(failure);
                    current.elapsed_ms = origin.elapsed().as_millis() as u64;
                    ipc_worker::report(&dir, &current).unwrap();
                    assert_eq!(behavior, "write-failure");
                }
            }
            "authorize" => {
                let dir = PathBuf::from(std::env::var_os("CLICKMASTER_FIXTURE_DIR").unwrap());
                let auth: ipc_worker::Authorization =
                    serde_json::from_reader(std::io::stdin()).unwrap();
                let _parent = ipc_worker::authorize(&dir, &auth).unwrap();
                assert!(ipc_worker::authorize(&dir, &auth).is_err());
                fs::write(dir.join("fixture-passed"), "authorized-once").unwrap();
            }
            _ => panic!("unknown fixture"),
        }
    }
    #[test]
    fn owned_child_cleanup_does_not_terminate_unrelated_child() {
        let mut owned = child("sleep").spawn().unwrap();
        let mut unrelated = child("sleep").spawn().unwrap();
        assert!(cleanup(&mut owned));
        assert!(owned.try_wait().unwrap().is_some());
        assert!(unrelated.try_wait().unwrap().is_none());
        assert!(cleanup(&mut unrelated));
    }

    #[test]
    fn persistent_pipe_equal_work_preserves_frames_with_one_worker_and_connection() {
        for interval in [100u64, 1] {
            let dir = fixture_dir(&format!("persistent-{}", native::random().unwrap()));
            let name = format!(
                r"\\.\pipe\ClickMaster-OfflineFixture-{}",
                native::random().unwrap()
            );
            let start = Instant::now();
            let mut worker = child("persistent-server")
                .env("CLICKMASTER_FIXTURE_PIPE", &name)
                .env("CLICKMASTER_FIXTURE_DIR", &dir)
                .spawn()
                .unwrap();
            let handle = crate::perf_native::duplicate(&worker);
            await_marker(&dir.join("ready"));
            let mut client = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(name)
                .unwrap();
            let reader = thread::spawn(move || {
                for _ in 0..24 {
                    let mut frame = [0u8; 4];
                    client.read_exact(&mut frame).unwrap();
                    assert_eq!(frame, ipc::FRAME);
                }
                client
            });
            let input = worker.stdin.take().unwrap();
            let mut session = PersistentSession {
                child: worker,
                input,
                dir: dir.clone(),
                sequence: 0,
            };
            let mut marks = Vec::new();
            for sequence in 1..=24u64 {
                session.input.write_all(&[1]).unwrap();
                session.input.flush().unwrap();
                let (outcome, receipt, clean) = session.wait(|| None, |_, _| {});
                assert_eq!(outcome, "completed");
                assert!(receipt.written && clean);
                assert_eq!(session.sequence, sequence);
                assert_eq!(receipt.read_status, ipc::ReadStatus::Unconfirmed);
                marks.push(start.elapsed().as_micros() as u64);
                if sequence != 24 {
                    thread::sleep(Duration::from_millis(interval));
                }
            }
            let _client = reader.join().unwrap();
            assert!(session.shutdown());
            let cpu = handle.as_ref().and_then(|h| crate::perf_native::cpu(h.0));
            assert!(session.child.try_wait().unwrap().is_some());
            let gaps: Vec<_> = marks.windows(2).map(|v| v[1] - v[0]).collect();
            let json = serde_json::json!({
                "scope":"non-game-persistent-protocol-fixture",
                "configuredIntervalMs":interval,"frames":24,"workers":1,"connections":1,
                "elapsedMicros":start.elapsed().as_micros() as u64,
                "interFrameMicros":gaps,"workerCpu100ns":cpu,
                "gameOrUiAcceptance":false,
            });
            fs::write(
                fixture_dir("persistent-summary").join(format!("interval-{interval}.json")),
                serde_json::to_vec_pretty(&json).unwrap(),
            )
            .unwrap();
        }
    }

    #[test]
    fn persistent_connection_failure_stops_without_reconnect_or_second_write() {
        let dir = fixture_dir(&format!("persistent-close-{}", native::random().unwrap()));
        let name = format!(
            r"\\.\pipe\ClickMaster-OfflineFixture-{}",
            native::random().unwrap()
        );
        let mut worker = child("persistent-server")
            .env("CLICKMASTER_FIXTURE_PIPE", &name)
            .env("CLICKMASTER_FIXTURE_DIR", &dir)
            .spawn()
            .unwrap();
        await_marker(&dir.join("ready"));
        let client = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(name)
            .unwrap();
        // Allow the server's single connect to finish before removing the peer.
        thread::sleep(Duration::from_millis(20));
        drop(client);
        worker.stdin.as_mut().unwrap().write_all(&[1]).unwrap();
        let start = Instant::now();
        while worker.try_wait().unwrap().is_none() && start.elapsed() < Duration::from_secs(2) {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(worker.try_wait().unwrap().is_some());
        assert!(cleanup(&mut worker));
        let report: Report =
            serde_json::from_slice(&fs::read(dir.join("status.json")).unwrap()).unwrap();
        assert_eq!(report.sequence, 1);
        assert_eq!(report.phase, "failed");
        assert!(!report.receipt.written);
        assert_eq!(
            report.receipt.failure.unwrap().stage,
            ipc::FailureStage::Write
        );
    }

    #[test]
    fn persistent_stale_sequence_cannot_hide_worker_exit_or_duplicate_sent_count() {
        let dir = fixture_dir(&format!("persistent-stale-{}", native::random().unwrap()));
        let mut worker = child("sleep").spawn().unwrap();
        let input = worker.stdin.take().unwrap();
        let mut previous = Report::new("completed");
        previous.sequence = 7;
        previous.receipt.connected = true;
        previous.receipt.written = true;
        ipc_worker::report(&dir, &previous).unwrap();
        assert!(cleanup(&mut worker));
        let mut session = PersistentSession {
            child: worker,
            input,
            dir,
            sequence: 7,
        };
        let start = Instant::now();
        let (outcome, receipt, clean) = session.wait(|| None, |_, _| {});
        assert_eq!(outcome, "failed");
        assert!(clean && !receipt.written);
        assert!(start.elapsed() < Duration::from_secs(1));
        let mut count = 7;
        assert!(ipc::record_sent(&mut count, &receipt));
        assert_eq!(count, 7);
    }

    #[test]
    fn persistent_unread_pipe_has_bounded_native_backpressure() {
        let dir = fixture_dir(&format!(
            "persistent-pressure-{}",
            native::random().unwrap()
        ));
        let name = format!(
            r"\\.\pipe\ClickMaster-OfflineFixture-{}",
            native::random().unwrap()
        );
        let mut worker = child("persistent-pressure")
            .env("CLICKMASTER_FIXTURE_PIPE", &name)
            .env("CLICKMASTER_FIXTURE_DIR", &dir)
            .spawn()
            .unwrap();
        await_marker(&dir.join("ready"));
        let _client = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(name)
            .unwrap();
        let start = Instant::now();
        while worker.try_wait().unwrap().is_none() && start.elapsed() < Duration::from_secs(5) {
            thread::sleep(Duration::from_millis(5));
        }
        let exited = worker.try_wait().unwrap().is_some_and(|s| s.success());
        assert!(cleanup(&mut worker));
        assert!(exited);
        let evidence: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("pressure.json")).unwrap()).unwrap();
        assert!(evidence["completeFramesWritten"].as_u64().unwrap() > 0);
        assert_eq!(evidence["consumerReadFrames"], 0);
        assert_eq!(evidence["reconnections"], 0);
    }
    #[test]
    fn non_game_pipe_enforces_first_instance_and_transfers_exact_single_frame() {
        let name = format!(
            r"\\.\pipe\ClickMaster-OfflineFixture-{}",
            native::random().unwrap()
        );
        assert_ne!(name, ipc::PIPE);
        let pipe = native::pipe(&name).unwrap();
        assert!(native::pipe(&name).is_err());
        let mut client = child("read")
            .env("CLICKMASTER_FIXTURE_PIPE", &name)
            .spawn()
            .unwrap();
        let result = (|| -> Result<(), String> {
            native::connect_bounded(&pipe, 2000)?;
            native::write(&pipe)?;
            native::drain(&pipe).map_err(|_| "fixture read failed".into())
        })();
        let deadline = Instant::now();
        while client.try_wait().unwrap().is_none() && deadline.elapsed() < Duration::from_secs(2) {
            thread::sleep(Duration::from_millis(10));
        }
        let success = client
            .try_wait()
            .unwrap()
            .is_some_and(|status| status.success());
        assert!(cleanup(&mut client));
        result.unwrap();
        assert!(success);
    }
    #[test]
    fn empty_fixture_connection_has_bounded_timeout() {
        let name = format!(
            r"\\.\pipe\ClickMaster-OfflineFixture-{}",
            native::random().unwrap()
        );
        let pipe = native::pipe(&name).unwrap();
        let now = Instant::now();
        assert!(native::connect_bounded(&pipe, 60).is_err());
        assert!(now.elapsed() < Duration::from_secs(2));
    }
    #[test]
    fn worker_authorization_is_private_parent_bound_and_one_time() {
        let root = fixture_dir("authorization");
        let (dir, token) = ipc_worker::attempt(&root).unwrap();
        let (_, parent_created, _) = native::process(std::process::id()).unwrap();
        let mut auth = ipc_worker::Authorization {
            diagnostics: 0,
            token,
            target: crate::targets::Identity {
                hwnd: 0,
                pid: 0,
                thread: 0,
                created: 0,
            },
            parent: std::process::id(),
            parent_created,
        };
        let real = auth.token.clone();
        auth.token = "A".repeat(64);
        assert!(ipc_worker::authorize(&dir, &auth).is_err());
        auth.token = real;
        let mut worker = child("authorize")
            .env("CLICKMASTER_FIXTURE_DIR", &dir)
            .spawn()
            .unwrap();
        worker
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(&auth).unwrap())
            .unwrap();
        let start = Instant::now();
        while worker.try_wait().unwrap().is_none() && start.elapsed() < Duration::from_secs(3) {
            thread::sleep(Duration::from_millis(10));
        }
        let success = worker.try_wait().unwrap().is_some_and(|s| s.success());
        assert!(cleanup(&mut worker));
        assert!(success);
        assert!(dir.join("fixture-passed").is_file());
        let manifest = fs::read_to_string(dir.join("authorization.json")).unwrap();
        assert!(!manifest.contains(&auth.token));
    }
    #[test]
    fn stopping_owned_worker_interrupts_native_blocked_drain() {
        blocked_drain(false);
    }
    #[test]
    fn deadline_interrupts_owned_worker_blocked_drain_with_fake_clock() {
        blocked_drain(true);
    }
    fn blocked_drain(expiry: bool) {
        let dir = fixture_dir(&format!("drain-{}", native::random().unwrap()));
        let name = format!(
            r"\\.\pipe\ClickMaster-OfflineFixture-{}",
            native::random().unwrap()
        );
        let mut worker = child("drain-server")
            .env("CLICKMASTER_FIXTURE_PIPE", &name)
            .env("CLICKMASTER_FIXTURE_DIR", &dir)
            .spawn()
            .unwrap();
        let start = Instant::now();
        while !dir.join("ready").exists() && start.elapsed() < Duration::from_secs(2) {
            thread::sleep(Duration::from_millis(10));
        }
        let client = fs::OpenOptions::new().read(true).write(true).open(name);
        let start = Instant::now();
        while !dir.join("draining").exists() && start.elapsed() < Duration::from_secs(2) {
            thread::sleep(Duration::from_millis(10));
        }
        let draining = dir.join("draining").exists();
        let still_waiting = worker.try_wait().unwrap().is_none();
        let mut clock = crate::duration::Clock::new(crate::duration::Policy::Timed(1));
        clock.begin(3000).unwrap();
        if expiry {
            assert_eq!(
                crate::duration::interruption("draining", false, clock.expired(4000)),
                Some("expired")
            );
        }
        let stopped = cleanup(&mut worker);
        drop(client);
        assert!(stopped);
        assert!(draining && still_waiting);
    }

    fn await_marker(path: &Path) {
        let start = Instant::now();
        while !path.exists() && start.elapsed() < Duration::from_secs(3) {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(path.exists(), "fixture handshake timed out");
    }
    fn observed_fixture(behavior: &str) -> (Child, PathBuf, fs::File) {
        let dir = fixture_dir(&format!("observe-{}", native::random().unwrap()));
        let name = format!(
            r"\\.\pipe\ClickMaster-OfflineFixture-{}",
            native::random().unwrap()
        );
        assert_ne!(name, ipc::PIPE);
        let worker = child("observed-server")
            .env("CLICKMASTER_FIXTURE_PIPE", &name)
            .env("CLICKMASTER_FIXTURE_DIR", &dir)
            .env("CLICKMASTER_FIXTURE_BEHAVIOR", behavior)
            .spawn()
            .unwrap();
        await_marker(&dir.join("ready"));
        let client = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(name)
            .unwrap();
        (worker, dir, client)
    }
    fn read_exact_one(mut client: fs::File, delay: u64) {
        thread::sleep(Duration::from_millis(delay));
        let mut frame = [0; 4];
        client.read_exact(&mut frame).unwrap();
        assert_eq!(frame, ipc::FRAME);
        let mut extra = [0; 1];
        // A closed server may return EOF or a broken pipe; neither is a second frame.
        assert!(!matches!(client.read(&mut extra), Ok(n) if n > 0));
    }
    #[test]
    fn timely_and_delayed_reads_preserve_exact_frame_and_confirm_diagnostics() {
        for delay in [0, 100] {
            let (mut worker, dir, client) = observed_fixture("normal");
            let reader = thread::spawn(move || read_exact_one(client, delay));
            let (outcome, receipt, clean) =
                supervise(&mut worker, &dir, Instant::now(), |_, _| None, |_, _| {});
            reader.join().unwrap();
            assert_eq!(outcome, "completed");
            assert!(receipt.written && clean);
            assert_eq!(receipt.read_status, ipc::ReadStatus::Confirmed);
            assert!(worker.try_wait().unwrap().is_some());
        }
    }
    #[test]
    fn unconfirmed_reads_have_bounded_cleanup_and_do_not_block_later_sends() {
        let mut sent = 0;
        for _ in 0..3 {
            let (mut worker, dir, mut client) = observed_fixture("normal");
            let start = Instant::now();
            let (outcome, receipt, clean) =
                supervise(&mut worker, &dir, start, |_, _| None, |_, _| {});
            assert_eq!(outcome, "completed");
            assert!(receipt.written && clean);
            assert_eq!(receipt.read_status, ipc::ReadStatus::Unconfirmed);
            assert!(start.elapsed() < Duration::from_secs(2));
            assert!(worker.try_wait().unwrap().is_some());
            assert!(ipc::record_sent(&mut sent, &receipt));
            // Check actual closure behavior without assuming that a successful write
            // guarantees consumption after the short observation window.
            let mut frame = [0; 4];
            let late = client.read_exact(&mut frame);
            if late.is_ok() {
                assert_eq!(frame, ipc::FRAME);
            }
            println!("post-cleanup buffered frame readable: {}", late.is_ok());
        }
        assert_eq!(sent, 3);
    }
    #[test]
    fn read_api_failure_is_diagnostic_but_native_write_failure_stops() {
        let (mut worker, dir, client) = observed_fixture("read-failure");
        let mut client = Some(client);
        let (outcome, receipt, clean) = supervise(
            &mut worker,
            &dir,
            Instant::now(),
            |phase, _| {
                if phase == "draining" && client.is_some() {
                    drop(client.take());
                    fs::write(dir.join("allow-drain"), "go").unwrap();
                }
                None
            },
            |_, _| {},
        );
        assert_eq!(outcome, "completed");
        assert!(clean && receipt.written);
        assert_eq!(receipt.read_status, ipc::ReadStatus::Failed);
        assert!(receipt.read_error_code.is_some());

        let (mut worker, dir, client) = observed_fixture("write-failure");
        await_marker(&dir.join("connected"));
        drop(client);
        fs::write(dir.join("allow-write"), "go").unwrap();
        let (outcome, receipt, clean) =
            supervise(&mut worker, &dir, Instant::now(), |_, _| None, |_, _| {});
        assert_eq!(outcome, "failed");
        assert!(clean && !receipt.written);
        let failure = receipt
            .failure
            .as_ref()
            .expect("native write failure survives worker exit");
        assert_eq!(failure.stage, ipc::FailureStage::Write);
        assert!(failure.error_code.is_some());
        let mut sent = 0;
        assert!(ipc::record_sent(&mut sent, &receipt));
        assert_eq!(sent, 0);
    }
    #[test]
    fn final_report_published_between_poll_and_exit_is_collected() {
        let (mut worker, dir, client) = observed_fixture("final-race");
        let reader = thread::spawn(move || read_exact_one(client, 0));
        let mut released = false;
        let (outcome, receipt, clean) = supervise(
            &mut worker,
            &dir,
            Instant::now(),
            |phase, _| {
                if phase == "draining" && !released {
                    released = true;
                    fs::write(dir.join("allow-final"), "go").unwrap();
                    await_marker(&dir.join("final-published"));
                    thread::sleep(Duration::from_millis(50));
                }
                None
            },
            |_, _| {},
        );
        reader.join().unwrap();
        assert!(released && clean && receipt.written);
        assert_eq!(outcome, "completed");
        assert_eq!(receipt.read_status, ipc::ReadStatus::Confirmed);
    }
    #[test]
    fn stop_deadline_and_target_loss_preserve_sent_count_after_write() {
        for reason in ["cancelled", "expired", "target-invalid"] {
            let (mut worker, dir, client) = observed_fixture("normal");
            let mut clock = crate::duration::Clock::new(crate::duration::Policy::Timed(1));
            clock.begin(3000).unwrap();
            let (outcome, receipt, clean) = supervise(
                &mut worker,
                &dir,
                Instant::now(),
                |phase, _| {
                    if phase != "draining" {
                        return None;
                    }
                    if reason == "target-invalid" {
                        return Some(reason);
                    }
                    crate::duration::interruption(
                        phase,
                        reason == "cancelled",
                        reason == "expired" && clock.expired(4000),
                    )
                },
                |_, _| {},
            );
            drop(client);
            assert_eq!(outcome, reason);
            assert!(clean && receipt.written);
            assert_eq!(receipt.read_status, ipc::ReadStatus::Unconfirmed);
            let mut sent = 0;
            assert!(ipc::record_sent(&mut sent, &receipt));
            assert_eq!(sent, 1);
            assert!(worker.try_wait().unwrap().is_some());
        }
    }
    #[test]
    fn stop_before_write_sends_nothing_and_does_not_count() {
        let (mut worker, dir, client) = observed_fixture("wait-write");
        let (outcome, receipt, clean) = supervise(
            &mut worker,
            &dir,
            Instant::now(),
            |phase, _| (phase == "writing").then_some("cancelled"),
            |_, _| {},
        );
        drop(client);
        assert_eq!(outcome, "cancelled");
        assert!(clean && !receipt.written);
        let mut sent = 0;
        assert!(ipc::record_sent(&mut sent, &receipt));
        assert_eq!(sent, 0);
    }
    #[test]
    fn report_preserves_first_fixed_failure_when_a_later_report_changes_it() {
        let dir = fixture_dir(&format!("failure-{}", native::random().unwrap()));
        let origin = Instant::now();
        let mut state = Observation::new(origin);
        let first =
            ipc::Failure::new(ipc::FailureStage::Connect, ipc::FailureReason::SystemCall).code(5);
        let mut report = Report::new("failed");
        report.receipt.fail(first.clone());
        ipc_worker::report(&dir, &report).unwrap();
        state.refresh(&dir, origin).unwrap();
        report.receipt.failure = Some(ipc::Failure::new(
            ipc::FailureStage::Cleanup,
            ipc::FailureReason::CleanupUnconfirmed,
        ));
        ipc_worker::report(&dir, &report).unwrap();
        state.refresh(&dir, origin).unwrap();
        assert_eq!(state.receipt.failure, Some(first));
        let text = serde_json::to_string(&state.receipt).unwrap();
        assert!(text.contains("connect") && text.contains("system-call") && text.contains('5'));
        assert!(!text.contains("pid") && !text.contains("target") && !text.contains("token"));
    }
    #[test]
    fn failure_after_complete_write_remains_failure_and_counts_the_sent_frame() {
        let (mut worker, dir, client) = observed_fixture("late-failure");
        let reader = thread::spawn(move || read_exact_one(client, 0));
        let (outcome, receipt, clean) =
            supervise(&mut worker, &dir, Instant::now(), |_, _| None, |_, _| {});
        reader.join().unwrap();
        assert_eq!(outcome, "failed");
        assert!(clean && receipt.written);
        assert_eq!(
            receipt.failure.as_ref().unwrap().stage,
            ipc::FailureStage::Status
        );
        assert_eq!(receipt.failure.as_ref().unwrap().error_code, Some(5));
        let mut sent = 0;
        assert!(ipc::record_sent(&mut sent, &receipt));
        assert_eq!(sent, 1);
    }
}
