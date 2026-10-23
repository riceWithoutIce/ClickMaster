//! Optional bounded telemetry. The input path never performs log IO or waits for a logger lock.
use crate::{ipc, ipc_native::Handle, perf_native as native, perf_states, perf_webview};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    os::windows::io::AsRawHandle,
    path::{Path, PathBuf},
    process::Child,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread,
    time::{Duration, Instant},
};

pub const LIMIT_MS: u64 = 60_000;
const SESSION_BYTES: usize = 1024 * 1024;
const FILE_BYTES: usize = SESSION_BYTES - 32 * 1024;
const KEEP: usize = 5;
const PREFIX: &str = "cmperf-";
const MAGIC: &str = "ClickMaster performance schema 2";
static ACTIVE: AtomicBool = AtomicBool::new(false);
static WORKER: AtomicBool = AtomicBool::new(false);
static DROPPED: AtomicU64 = AtomicU64::new(0);
static INPUT_LOST: AtomicU64 = AtomicU64::new(0);
static WINDOW_LOST: AtomicU64 = AtomicU64::new(0);
static GLOBAL: OnceLock<Arc<Collector>> = OnceLock::new();
static WORKER_METRICS: Mutex<Metrics> = Mutex::new(Metrics::empty());

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Stage {
    Spawn,
    Authorization,
    ParentExeHash,
    WorkerExeHash,
    AssemblyHash,
    FullValidation,
    Connect,
    Write,
    ReadObservation,
    Cleanup,
}
const LABELS: [&str; 10] = [
    "worker-spawn",
    "authorization",
    "parent-exe-hash",
    "worker-exe-hash",
    "assembly-hash",
    "full-validation",
    "connect",
    "write",
    "read-observation",
    "cleanup",
];
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StageStat {
    pub count: u64,
    pub total_micros: u64,
    pub max_micros: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metrics {
    pub stages: [StageStat; 10],
}
impl Metrics {
    const fn empty() -> Self {
        Self {
            stages: [StageStat {
                count: 0,
                total_micros: 0,
                max_micros: 0,
            }; 10],
        }
    }
    fn add(&mut self, stage: usize, micros: u64) {
        let s = &mut self.stages[stage];
        s.count = s.count.saturating_add(1);
        s.total_micros = s.total_micros.saturating_add(micros);
        s.max_micros = s.max_micros.max(micros);
    }
    fn merge(&mut self, other: &Self) {
        for (a, b) in self.stages.iter_mut().zip(other.stages) {
            a.count = a.count.saturating_add(b.count);
            a.total_micros = a.total_micros.saturating_add(b.total_micros);
            a.max_micros = a.max_micros.max(b.max_micros);
        }
    }
    pub fn valid(&self) -> bool {
        self.stages.iter().all(|s| {
            s.count <= 100_000 && s.total_micros <= 600_000_000 && s.max_micros <= s.total_micros
        })
    }
}
pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}
pub fn epoch() -> u64 {
    if active() {
        global().epoch.load(Ordering::Relaxed)
    } else {
        0
    }
}
pub fn set_worker(enabled: bool) {
    WORKER.store(enabled, Ordering::Relaxed);
}
pub fn worker_metrics() -> Option<Metrics> {
    WORKER
        .load(Ordering::Relaxed)
        .then(|| WORKER_METRICS.try_lock().ok().map(|m| m.clone()))
        .flatten()
}
pub struct Span(Option<(Instant, u64, bool)>, usize);
pub fn span(stage: Stage) -> Span {
    let worker = WORKER.load(Ordering::Relaxed);
    if !worker && !active() {
        return Span(None, stage as usize);
    }
    Span(
        Some((
            Instant::now(),
            global().epoch.load(Ordering::Relaxed),
            worker,
        )),
        stage as usize,
    )
}
impl Drop for Span {
    fn drop(&mut self) {
        let Some((at, epoch, worker)) = self.0 else {
            return;
        };
        let micros = at.elapsed().as_micros().min(u64::MAX as u128) as u64;
        if worker {
            if let Ok(mut m) = WORKER_METRICS.try_lock() {
                m.add(self.1, micros);
            }
        } else if active() && global().epoch.load(Ordering::Relaxed) == epoch {
            if let Ok(mut live) = global().live.try_lock() {
                live.parent_stages.add(self.1, micros);
            } else {
                DROPPED.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub active: bool,
    pub elapsed_seconds: u64,
    pub message: String,
    pub file_stem: Option<String>,
    pub complete: bool,
}
impl Default for View {
    fn default() -> Self {
        Self {
            active: false,
            elapsed_seconds: 0,
            message: "性能采集默认关闭。".into(),
            file_stem: None,
            complete: false,
        }
    }
}
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Context {
    running: Option<bool>,
    phase: &'static str,
    channel: &'static str,
    min_interval_ms: Option<u64>,
    max_interval_ms: Option<u64>,
    limit_seconds: Option<u64>,
    sent: u64,
    task_generation: u64,
    failure: Option<ipc::Failure>,
}
fn phase(value: &str) -> &'static str {
    match value {
        "idle" => "idle",
        "countdown" => "countdown",
        "checking" => "checking",
        "connecting" => "connecting",
        "writing" => "writing",
        "draining" => "draining",
        "waiting" => "waiting",
        "completed" => "completed",
        "cancelled" => "cancelled",
        "expired" => "expired",
        "stopping" => "stopping",
        "target-invalid" => "target-invalid",
        "failed" => "failed",
        "timeout" => "timeout",
        "cleanup-failed" => "cleanup-failed",
        "partial" => "partial",
        "rejected" => "rejected",
        _ => "unknown",
    }
}
pub fn context(status: &crate::input::Status) {
    if !active() {
        return;
    }
    if let Ok(mut live) = global().live.try_lock() {
        let running = (phase(&status.phase) != "unknown").then_some(status.busy);
        let input_loss = INPUT_LOST.load(Ordering::Relaxed);
        if live.context.running != running
            || live.context.task_generation != status.task_id
            || live.input_loss != input_loss
        {
            live.state_revision = live.state_revision.saturating_add(1);
        }
        live.input_loss = input_loss;
        live.context.running = running;
        let delta = if live.context.task_generation == status.task_id {
            status.completed.saturating_sub(live.context.sent)
        } else {
            status.completed
        };
        live.sent_observed = live.sent_observed.saturating_add(delta);
        live.context.phase = phase(&status.phase);
        live.context.channel = if status.channel == crate::channels::IPC {
            "ipc"
        } else {
            "message"
        };
        live.context.sent = status.completed;
        live.context.task_generation = status.task_id;
        live.context.limit_seconds = status.timing.limit_seconds;
        live.context.failure = status.ipc.as_ref().and_then(|r| r.failure.clone());
    } else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
        INPUT_LOST.fetch_add(1, Ordering::Relaxed);
    }
}
pub fn parameters(range: Option<crate::repeat::Range>) {
    if active() {
        if let Ok(mut live) = global().live.try_lock() {
            live.context.min_interval_ms = range.map(|r| r.min);
            live.context.max_interval_ms = range.map(|r| r.max);
        } else {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
    }
}
pub fn window_state(visible: Option<bool>, minimized: Option<bool>) {
    window_state_epoch(visible, minimized, None);
}
pub fn window_state_epoch(visible: Option<bool>, minimized: Option<bool>, epoch: Option<u64>) {
    if active() {
        if let Ok(mut live) = global().live.try_lock() {
            if epoch.is_some_and(|e| e != global().epoch.load(Ordering::Acquire)) {
                return;
            }
            let window = perf_states::window(visible, minimized);
            let window_loss = WINDOW_LOST.load(Ordering::Relaxed);
            if live.window != window || live.window_loss != window_loss {
                live.state_revision = live.state_revision.saturating_add(1);
            }
            live.window = window;
            live.window_loss = window_loss;
        } else {
            DROPPED.fetch_add(1, Ordering::Relaxed);
            WINDOW_LOST.fetch_add(1, Ordering::Relaxed);
        }
    }
}
struct Worker {
    number: u64,
    handle: Handle,
    base: Option<u64>,
    last: Option<u64>,
    observed: Instant,
    clipped_start: bool,
    collected: bool,
}
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Workers {
    observed: u64,
    spawned_in_capture: u64,
    exited: u64,
    cpu_ticks: u64,
    cpu_missing: u64,
    clipped_start: u64,
    clipped_end: u64,
    lifecycle_observed_micros: u64,
    reports: u64,
    incomplete_reports: u64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkerRecord {
    number: u64,
    cpu_seconds: Option<f64>,
    observed_micros: u64,
    clipped_start: bool,
    clipped_end: bool,
}
struct Live {
    context: Context,
    window: &'static str,
    parent_stages: Metrics,
    worker_stages: Metrics,
    children: BTreeMap<usize, Worker>,
    workers: Workers,
    sent_observed: u64,
    records: Vec<WorkerRecord>,
    state_revision: u64,
    input_loss: u64,
    window_loss: u64,
}
impl Default for Live {
    fn default() -> Self {
        Self {
            context: Context::default(),
            window: "unknown",
            parent_stages: Metrics::empty(),
            worker_stages: Metrics::empty(),
            children: BTreeMap::new(),
            workers: Workers::default(),
            sent_observed: 0,
            records: Vec::new(),
            state_revision: 0,
            input_loss: INPUT_LOST.load(Ordering::Relaxed),
            window_loss: WINDOW_LOST.load(Ordering::Relaxed),
        }
    }
}
impl Live {
    fn state(&self) -> perf_states::State {
        let input_loss = INPUT_LOST.load(Ordering::Relaxed);
        let window_loss = WINDOW_LOST.load(Ordering::Relaxed);
        perf_states::State {
            running: if input_loss == self.input_loss {
                self.context.running
            } else {
                None
            },
            window: if window_loss == self.window_loss {
                self.window
            } else {
                "unknown"
            },
            revision: self.state_revision,
            lost: input_loss.saturating_add(window_loss),
        }
    }
}
pub struct Collector {
    epoch: AtomicU64,
    stop: AtomicBool,
    exiting: AtomicBool,
    busy: AtomicBool,
    live: Mutex<Live>,
    view: Mutex<View>,
}
impl Default for Collector {
    fn default() -> Self {
        Self {
            epoch: AtomicU64::new(0),
            stop: AtomicBool::new(false),
            exiting: AtomicBool::new(false),
            busy: AtomicBool::new(false),
            live: Mutex::new(Live::default()),
            view: Mutex::new(View::default()),
        }
    }
}
fn global() -> &'static Arc<Collector> {
    GLOBAL.get_or_init(|| Arc::new(Collector::default()))
}

pub fn observe_child(child: &Child, spawned: bool) {
    if !active() {
        return;
    }
    let Ok(mut live) = global().live.try_lock() else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
        return;
    };
    let key = child.as_raw_handle() as usize;
    if spawned
        && live
            .children
            .get(&key)
            .is_some_and(|w| w.collected || native::exited(&w.handle))
    {
        if let Some(mut worker) = live.children.remove(&key) {
            if let Some(record) = collect_worker(&mut worker, &mut live.workers, true) {
                push_worker(&mut live.records, record);
            }
        }
    }
    if live.children.contains_key(&key) {
        return;
    }
    if live.children.len() >= 32 {
        DROPPED.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let Some(handle) = native::duplicate(child) else {
        live.workers.cpu_missing += 1;
        return;
    };
    let cpu = native::cpu(handle.0);
    live.workers.observed += 1;
    live.workers.spawned_in_capture += u64::from(spawned);
    live.workers.clipped_start += u64::from(!spawned);
    let number = live.workers.observed;
    live.children.insert(
        key,
        Worker {
            number,
            handle,
            base: if spawned { Some(0) } else { cpu },
            last: if spawned { Some(0) } else { cpu },
            observed: Instant::now(),
            clipped_start: !spawned,
            collected: false,
        },
    );
}
fn push_worker(records: &mut Vec<WorkerRecord>, record: WorkerRecord) {
    if records.len() < 1024 {
        records.push(record);
    } else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}
fn collect_worker(
    worker: &mut Worker,
    totals: &mut Workers,
    final_capture: bool,
) -> Option<WorkerRecord> {
    if worker.collected {
        return None;
    }
    let now = native::cpu(worker.handle.0);
    if let Some(delta) = advance_cpu(&mut worker.last, worker.base, now) {
        totals.cpu_ticks = totals.cpu_ticks.saturating_add(delta);
    } else {
        totals.cpu_missing += 1;
    }
    let exited = native::exited(&worker.handle);
    if exited || final_capture {
        totals.exited += u64::from(exited);
        totals.clipped_end += u64::from(!exited);
        totals.lifecycle_observed_micros = totals
            .lifecycle_observed_micros
            .saturating_add(worker.observed.elapsed().as_micros() as u64);
        worker.collected = true;
        return Some(WorkerRecord {
            number: worker.number,
            cpu_seconds: worker
                .base
                .zip(now)
                .filter(|(a, b)| b >= a)
                .map(|(a, b)| (b - a) as f64 / 10_000_000.0),
            observed_micros: worker.observed.elapsed().as_micros() as u64,
            clipped_start: worker.clipped_start,
            clipped_end: !exited,
        });
    }
    None
}
fn advance_cpu(last: &mut Option<u64>, base: Option<u64>, now: Option<u64>) -> Option<u64> {
    let delta = last.or(base).zip(now).and_then(|(a, b)| b.checked_sub(a));
    // Preserve the last successful baseline across a query failure, avoiding double count.
    if now.is_some() {
        *last = now;
    }
    delta
}
pub fn finish_child(child: &Child) {
    if !active() {
        return;
    }
    // A short contention must not lose the final CPU reading: retained duplicates are also sampled.
    if let Ok(mut live) = global().live.try_lock() {
        let key = child.as_raw_handle() as usize;
        if let Some(mut worker) = live.children.remove(&key) {
            if let Some(record) = collect_worker(&mut worker, &mut live.workers, true) {
                push_worker(&mut live.records, record);
            }
        }
    } else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}
pub fn merge_worker(metrics: Option<&Metrics>, clipped: bool, report_epoch: u64) {
    if !active() {
        return;
    }
    if let Ok(mut live) = global().live.try_lock() {
        if let Some(metrics) =
            metrics.filter(|m| m.valid() && report_epoch != 0 && report_epoch == epoch())
        {
            live.worker_stages.merge(metrics);
            live.workers.reports += 1;
        } else {
            live.workers.incomplete_reports += 1;
        }
        if clipped && metrics.is_some() {
            live.workers.incomplete_reports += 1;
        }
    } else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn view() -> View {
    global()
        .view
        .lock()
        .map(|v| v.clone())
        .unwrap_or_else(|_| View {
            message: "性能采集状态不可用。".into(),
            ..Default::default()
        })
}
#[cfg(test)]
pub fn start(
    root: PathBuf,
    initial: crate::input::Status,
    range: Option<crate::repeat::Range>,
) -> Result<View, String> {
    start_impl(root, initial, range, None)
}
pub fn start_with_window(
    root: PathBuf,
    initial: crate::input::Status,
    range: Option<crate::repeat::Range>,
    window: Option<tauri::WebviewWindow>,
) -> Result<View, String> {
    start_impl(root, initial, range, window.map(perf_webview::Source::new))
}
fn start_impl(
    root: PathBuf,
    initial: crate::input::Status,
    range: Option<crate::repeat::Range>,
    source: Option<perf_webview::Source>,
) -> Result<View, String> {
    let collector = global().clone();
    if collector
        .busy
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_err()
    {
        return Err("已有性能采集正在进行或收尾。".into());
    }
    collector.stop.store(false, Ordering::Relaxed);
    collector.exiting.store(false, Ordering::Relaxed);
    *collector.live.lock().map_err(|_| "性能状态不可用。")? = Live::default();
    *collector.view.lock().map_err(|_| "性能采集状态不可用。")? = View {
        active: true,
        message: "正在准备性能采集…".into(),
        ..Default::default()
    };
    let copy = collector.clone();
    if thread::Builder::new()
        .name("performance-capture".into())
        .spawn(move || {
            if let Err(message) = capture(&copy, &root, LIMIT_MS, &initial, range, source.as_ref())
            {
                if let Ok(mut v) = copy.view.lock() {
                    v.active = false;
                    v.complete = false;
                    v.message = message.into();
                }
            }
            ACTIVE.store(false, Ordering::Release);
            if let Ok(mut live) = copy.live.lock() {
                live.children.clear();
            }
            copy.busy.store(false, Ordering::Release);
        })
        .is_err()
    {
        collector.busy.store(false, Ordering::Release);
        if let Ok(mut v) = collector.view.lock() {
            v.active = false;
            v.message = "无法启动性能采集。".into();
        }
        return Err("无法启动性能采集。".into());
    }
    Ok(view())
}
pub fn stop() -> View {
    global().stop.store(true, Ordering::Release);
    view()
}
pub fn shutdown() {
    global().exiting.store(true, Ordering::Release);
    global().stop.store(true, Ordering::Release);
    let at = Instant::now();
    while global().busy.load(Ordering::Acquire) && at.elapsed() < Duration::from_millis(250) {
        thread::sleep(Duration::from_millis(5));
    }
}

fn end_reason(elapsed_ms: u64, limit: u64, stop: bool, exiting: bool) -> Option<&'static str> {
    if stop {
        Some(if exiting { "exit" } else { "manual" })
    } else if elapsed_ms >= limit {
        Some("duration")
    } else {
        None
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    kind: String,
    magic: String,
    sequence: u64,
    schema: u32,
    version: String,
    logical_processors: usize,
    limit_ms: u64,
    webview: String,
    limitations: Vec<String>,
}
fn managed_stem(value: &str) -> bool {
    value
        .strip_prefix(PREFIX)
        .is_some_and(crate::ipc_worker::valid_run)
}
struct Files {
    json: File,
    text: File,
    stem: String,
    sequence: u64,
    bytes: usize,
    buffer: Vec<u8>,
    _lock: Handle,
}
impl Files {
    fn prepare(root: &Path) -> Result<Self, &'static str> {
        let dir = root.join("logs");
        fs::create_dir_all(&dir).map_err(|_| "无法创建性能日志目录；未改变连点状态。")?;
        if fs::symlink_metadata(&dir)
            .map_err(|_| "无法检查日志目录。")?
            .file_type()
            .is_symlink()
        {
            return Err("性能日志目录不支持链接。");
        }
        let lock = native::directory_lock(&dir.join(".clickmaster-performance.lock"))
            .ok_or("日志目录正由其他实例使用，或不可写；未开始采集。")?;
        let mut items: Vec<(u64, PathBuf, PathBuf, u64)> = Vec::new();
        let mut reserved_bytes = 0u64;
        let mut reserved_sets = 0usize;
        let mut sequence = 0u64;
        let mut stems = std::collections::BTreeSet::new();
        for entry in fs::read_dir(&dir).map_err(|_| "无法读取日志预算。")? {
            let entry = entry.map_err(|_| "无法读取日志预算。")?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let Some(stem) = name
                .strip_suffix(".jsonl")
                .or_else(|| name.strip_suffix(".txt"))
            else {
                continue;
            };
            if !managed_stem(stem) {
                continue;
            }
            let meta = fs::symlink_metadata(entry.path()).map_err(|_| "无法检查日志文件。")?;
            if !meta.is_file() || meta.file_type().is_symlink() {
                return Err("日志目录含不可安全处理的记录。");
            }
            reserved_bytes = reserved_bytes.saturating_add(meta.len());
            stems.insert(stem.to_string());
        }
        reserved_sets += stems.len();
        for stem in stems {
            let json = dir.join(format!("{stem}.jsonl"));
            let text = dir.join(format!("{stem}.txt"));
            let len = fs::metadata(&json).map(|m| m.len()).unwrap_or(0)
                + fs::metadata(&text).map(|m| m.len()).unwrap_or(0);
            if len > SESSION_BYTES as u64 {
                continue;
            }
            let (Ok(bytes), Ok(summary)) = (fs::read(&json), fs::read_to_string(&text)) else {
                continue;
            };
            let mut lines = bytes.split(|b| *b == b'\n').filter(|s| !s.is_empty());
            let Some(first) = lines.next() else {
                continue;
            };
            let Ok(header) = serde_json::from_slice::<Header>(first) else {
                continue;
            };
            if !matches!(
                (header.schema, header.magic.as_str()),
                (1, "ClickMaster performance schema 1") | (2, MAGIC)
            ) || header.kind != "header"
            {
                continue;
            }
            sequence = sequence.max(header.sequence);
            let complete = lines
                .last()
                .and_then(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
                .is_some_and(|v| {
                    v["kind"] == "summary" && v["complete"] == true && v["magic"] == header.magic
                });
            if complete && summary.starts_with(&header.magic) {
                items.push((header.sequence, json, text, len));
            }
        }
        items.sort_by_key(|i| i.0);
        for (_, json, text, len) in items {
            if reserved_sets < KEEP
                && reserved_bytes + SESSION_BYTES as u64 <= (KEEP * SESSION_BYTES) as u64
            {
                break;
            }
            // Exclusive handles are acquired before either delete; active/locked pairs are preserved.
            let Some(json_lock) = exclusive_existing(&json) else {
                continue;
            };
            let Some(text_lock) = exclusive_existing(&text) else {
                drop(json_lock);
                continue;
            };
            drop(json_lock);
            drop(text_lock);
            // Directory ownership excludes other conforming writers; never follow directory links.
            fs::remove_file(&json).map_err(|_| "无法轮换已完成日志；未开始采集。")?;
            fs::remove_file(&text).map_err(|_| "无法轮换已完成日志；未开始采集。")?;
            reserved_sets -= 1;
            reserved_bytes = reserved_bytes.saturating_sub(len);
        }
        if reserved_sets >= KEEP
            || reserved_bytes + SESSION_BYTES as u64 > (KEEP * SESSION_BYTES) as u64
        {
            return Err("日志保留预算不足；未知或不完整记录已保留。");
        }
        let stem = format!(
            "{PREFIX}{}",
            crate::ipc_native::random().map_err(|_| "无法创建匿名日志编号。")?
        );
        let json = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(format!("{stem}.jsonl")))
            .map_err(|_| "无法创建性能日志。")?;
        let text = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(format!("{stem}.txt")))
            .map_err(|_| "无法创建性能摘要。")?;
        Ok(Self {
            json,
            text,
            stem,
            sequence: sequence.checked_add(1).ok_or("日志序号已达上限。")?,
            bytes: 0,
            buffer: Vec::with_capacity(16 * 1024),
            _lock: lock,
        })
    }
    fn line(&mut self, value: &impl Serialize) -> Result<(), &'static str> {
        let mut bytes = serde_json::to_vec(value).map_err(|_| "性能记录编码失败。")?;
        bytes.push(b'\n');
        if self.bytes + self.buffer.len() + bytes.len() > FILE_BYTES {
            return Err("性能采集达到文件预算。");
        }
        self.buffer.extend(bytes);
        Ok(())
    }
    fn flush(&mut self) -> Result<(), &'static str> {
        self.json
            .write_all(&self.buffer)
            .map_err(|_| "性能日志写入失败；记录可能不完整。")?;
        self.bytes += self.buffer.len();
        self.buffer.clear();
        Ok(())
    }
}
fn exclusive_existing(path: &Path) -> Option<Handle> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{Foundation::GENERIC_READ, Storage::FileSystem::*};
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    Handle::new(unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ,
            0,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    })
    .ok()
}

pub fn cpu_delta(
    previous: Option<u64>,
    now: Option<u64>,
    elapsed: f64,
    processors: usize,
) -> (Option<f64>, Option<f64>) {
    match previous
        .zip(now)
        .filter(|(a, b)| b >= a)
        .filter(|_| elapsed > 0.0 && processors > 0)
    {
        Some((a, b)) => {
            let seconds = (b - a) as f64 / 10_000_000.0;
            (
                Some(seconds),
                Some(seconds / elapsed / processors as f64 * 100.0),
            )
        }
        None => (None, None),
    }
}
fn capture(
    collector: &Collector,
    root: &Path,
    limit: u64,
    initial: &crate::input::Status,
    range: Option<crate::repeat::Range>,
    source: Option<&perf_webview::Source>,
) -> Result<(), &'static str> {
    let mut files = Files::prepare(root)?;
    let processors = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);
    files.line(&Header {
        kind: "header".into(),
        magic: MAGIC.into(),
        sequence: files.sequence,
        schema: 2,
        version: env!("CARGO_PKG_VERSION").into(),
        logical_processors: processors,
        limit_ms: limit,
        webview: "webview-environment; sharedPossible=true; same user data folder; excludes crashpad; not EXE-exclusive".into(),
        limitations: vec![
            "wall time stages overlap; do not sum as CPU".into(),
            "worker stages may be partial when worker is interrupted".into(),
            "WebView2 environment may be shared across instances/applications; missing/stale data is not zero".into(),
            "working-set sums can overlap/shared; not exclusive physical memory".into(),
            "memory sampled at one second; peaks may be missed".into(),
        ],
    })?;
    DROPPED.store(0, Ordering::Relaxed);
    let capture_epoch = collector.epoch.fetch_add(1, Ordering::AcqRel) + 1;
    let origin = Instant::now();
    let mut previous_at = origin;
    let mut previous_cpu = native::current_cpu();
    let initial_cpu = previous_cpu;
    let mut last_sent = 0u64;
    let mut last_workers = 0;
    let mut samples = 0u64;
    let mut main_cpu_total = 0f64;
    let mut main_cpu_missing = 0u64;
    let mut webview_sampler = perf_webview::Sampler::default();
    let mut comparison = perf_states::Comparison::default();
    let mut previous_state;
    {
        let mut live = collector.live.lock().map_err(|_| "性能状态不可用。")?;
        live.context = Context {
            running: (phase(&initial.phase) != "unknown").then_some(initial.busy),
            phase: phase(&initial.phase),
            channel: if initial.channel == crate::channels::IPC {
                "ipc"
            } else {
                "message"
            },
            min_interval_ms: range.map(|r| r.min),
            max_interval_ms: range.map(|r| r.max),
            limit_seconds: initial.timing.limit_seconds,
            sent: initial.completed,
            task_generation: initial.task_id,
            failure: initial.ipc.as_ref().and_then(|r| r.failure.clone()),
        };
        live.sent_observed = 0;
        if let Some(source) = source {
            live.window = perf_states::window(source.initial_window.0, source.initial_window.1);
        }
        previous_state = live.state();
    }
    ACTIVE.store(true, Ordering::Release);
    if let Some(source) = source {
        source.request(capture_epoch);
    }
    if let Ok(mut v) = collector.view.lock() {
        *v = View {
            active: true,
            file_stem: Some(files.stem.clone()),
            message: "采集中；60秒后自动结束，不改变连点状态。".into(),
            ..Default::default()
        };
    }
    let mut next_sample = Duration::from_secs(1);
    let mut next_flush = Duration::from_secs(5);
    let reason = loop {
        if let Some(reason) = end_reason(
            origin.elapsed().as_millis() as u64,
            limit,
            collector.stop.load(Ordering::Acquire),
            collector.exiting.load(Ordering::Relaxed),
        ) {
            break reason;
        }
        if origin.elapsed() < next_sample {
            thread::sleep(Duration::from_millis(20));
            continue;
        }
        let now = Instant::now();
        let cpu = native::current_cpu();
        let seconds = now.duration_since(previous_at).as_secs_f64();
        let (cpu_seconds, cpu_percent) = cpu_delta(previous_cpu, cpu, seconds, processors);
        previous_cpu = cpu;
        previous_at = now;
        if let Some(cpu) = cpu_seconds {
            main_cpu_total += cpu;
        } else {
            main_cpu_missing += 1;
        }
        let memory = native::memory();
        let webview = webview_sampler.sample(source, seconds, processors);
        let mut live = collector.live.lock().map_err(|_| "性能状态不可用。")?;
        let Live {
            children,
            workers,
            records,
            ..
        } = &mut *live;
        for worker in children.values_mut() {
            if let Some(record) = collect_worker(worker, workers, false) {
                push_worker(records, record);
            }
        }
        let workers_delta = live.workers.cpu_ticks.saturating_sub(last_workers);
        last_workers = live.workers.cpu_ticks;
        let workers_cpu = if live.workers.cpu_missing == 0 {
            Some(workers_delta as f64 / 10_000_000.0)
        } else {
            None
        };
        let sent = live.sent_observed.saturating_sub(last_sent);
        last_sent = live.sent_observed;
        let state = live.state();
        let bin = perf_states::group(previous_state, state);
        previous_state = state;
        comparison.add(
            bin,
            seconds,
            sent,
            cpu_seconds,
            workers_cpu,
            memory,
            &webview,
            processors,
        );
        let record = serde_json::json!({"kind":"sample","relativeMs":origin.elapsed().as_millis() as u64,"intervalSeconds":seconds,"intervalState":perf_states::Comparison::name(bin),"context":live.context,"window":live.window,"main":{"cpuSeconds":cpu_seconds,"machineCpuPercent":cpu_percent,"workingSetBytes":memory.map(|m|m.0),"privateBytes":memory.map(|m|m.1)},"workers":{"cpuSeconds":workers_cpu,"machineCpuPercent":if processors>0 {workers_cpu.map(|cpu|cpu/seconds/processors as f64*100.0)}else{None},"tickUnit":"100ns","totals":live.workers},"webview":webview,"sentDelta":sent,"sendsPerSecond":sent as f64/seconds});
        drop(live);
        if let Some(source) = source {
            source.request(capture_epoch);
        }
        if files.line(&record).is_err() {
            break "file-budget";
        }
        samples += 1;
        if let Ok(mut v) = collector.view.lock() {
            v.elapsed_seconds = origin.elapsed().as_secs();
        }
        next_sample = origin.elapsed() + Duration::from_secs(1);
        if origin.elapsed() >= next_flush {
            files.flush()?;
            next_flush = origin.elapsed() + Duration::from_secs(5);
        }
    };
    ACTIVE.store(false, Ordering::Release);
    if let Some(source) = source {
        source.cancel();
    }
    let elapsed = origin.elapsed().as_secs_f64();
    let (tail, _) = cpu_delta(
        previous_cpu,
        native::current_cpu(),
        Instant::now().duration_since(previous_at).as_secs_f64(),
        processors,
    );
    if let Some(tail) = tail {
        main_cpu_total += tail;
    } else {
        main_cpu_missing += 1;
    }
    let mut live = collector.live.lock().map_err(|_| "性能状态不可用。")?;
    let Live {
        children,
        workers,
        records,
        ..
    } = &mut *live;
    for worker in children.values_mut() {
        if let Some(record) = collect_worker(worker, workers, true) {
            push_worker(records, record);
        }
    }
    children.clear();
    let sent_total = live.sent_observed;
    files.line(&serde_json::json!({"kind":"worker-lifetimes","workers":live.records,"cpuUnit":"seconds","timeUnit":"microseconds","coverage":"observed capture interval; clippedStart excludes earlier CPU"}))?;
    let stages = serde_json::json!({"kind":"stages","labels":LABELS,"unit":"wall-clock-microseconds","nested":true,"parent":live.parent_stages,"worker":live.worker_stages,"workerCoverage":"reports only; interrupted/read-observation stages may be partial"});
    files.line(&stages)?;
    let summary = serde_json::json!({"kind":"summary","magic":MAGIC,"complete":true,"reason":reason,"elapsedSeconds":elapsed,"samples":samples,"mainCpuSecondsObserved":main_cpu_total,"mainCpuMissingIntervals":main_cpu_missing,"mainInitialCpuAvailable":initial_cpu.is_some(),"mainMachineCpuPercentObserved":if processors>0&&elapsed>0.0{Some(main_cpu_total/elapsed/processors as f64*100.0)}else{None},"workers":live.workers,"sentObserved":sent_total,"droppedMeasurements":DROPPED.load(Ordering::Relaxed),"lastContext":live.context,"window":live.window,"webviewScope":"webview-environment; sharedPossible=true; excludes crashpad; not EXE-exclusive","stateComparison":comparison.json(),"limitations":"partial CPU/worker stages and one-second memory sampling; not a CPU profiler"});
    let text=format!("{MAGIC}\n版本：{}\n结束：{reason}；采集：{elapsed:.3} 秒；样本：{samples}\n主程序CPU（已观察）：{main_cpu_total:.6} 秒；缺失区间：{main_cpu_missing}\nworker CPU（已观察）：{:.6} 秒；观察/创建/退出：{}/{}/{}\nworker起/末截断：{}/{}；CPU缺失：{}；不完整阶段报告：{}\n已观察发送：{sent_total}；测量丢弃：{}\nWebView2：仅关联环境，可能共享、排除crashpad；资源缺失不是0，不等于EXE独占。\n阶段为嵌套墙钟耗时，不能相加当作CPU；详细参数/样本/阶段见同名JSONL。\n内存每秒快照可能漏峰值；边界及中断worker阶段不完整。\n",env!("CARGO_PKG_VERSION"),live.workers.cpu_ticks as f64/10_000_000.0,live.workers.observed,live.workers.spawned_in_capture,live.workers.exited,live.workers.clipped_start,live.workers.clipped_end,live.workers.cpu_missing,live.workers.incomplete_reports,DROPPED.load(Ordering::Relaxed));
    let mut text = text;
    text.push_str(&comparison.text());
    text.push_str(&format!(
        "参数：{}；间隔：{:?}～{:?} ms；限时秒数：{:?}（空为永久）；界面：{}\n",
        live.context.channel,
        live.context.min_interval_ms,
        live.context.max_interval_ms,
        live.context.limit_seconds,
        live.window
    ));
    for (label, (parent, worker)) in LABELS.iter().zip(
        live.parent_stages
            .stages
            .iter()
            .zip(live.worker_stages.stages.iter()),
    ) {
        text.push_str(&format!(
            "阶段 {label}：父{}次/{:.3}ms（最大{:.3}ms），子{}次/{:.3}ms（最大{:.3}ms）\n",
            parent.count,
            parent.total_micros as f64 / 1000.0,
            parent.max_micros as f64 / 1000.0,
            worker.count,
            worker.total_micros as f64 / 1000.0,
            worker.max_micros as f64 / 1000.0
        ));
    }
    drop(live);
    files.line(&summary)?;
    if files.bytes + files.buffer.len() + text.len() > SESSION_BYTES {
        return Err("性能采集达到文件预算，摘要可能不完整。");
    }
    files.flush()?;
    files
        .text
        .write_all(text.as_bytes())
        .map_err(|_| "性能摘要写入失败；记录可能不完整。")?;
    if let Ok(mut v) = collector.view.lock() {
        v.active = false;
        v.complete = true;
        v.elapsed_seconds = elapsed as u64;
        v.message = "采集已结束；请提供data/logs/中同名JSONL和TXT。".into();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root(name: &str) -> PathBuf {
        PathBuf::from(std::env::var_os("CLICKMASTER_TEST_ROOT").unwrap()).join(name)
    }
    #[test]
    fn cpu_math_distinguishes_missing_and_uses_real_interval() {
        let mut previous = Some(0);
        let mut total = advance_cpu(&mut previous, Some(0), Some(10)).unwrap();
        assert_eq!(advance_cpu(&mut previous, Some(0), None), None);
        assert_eq!(previous, Some(10));
        total += advance_cpu(&mut previous, Some(0), Some(30)).unwrap();
        assert_eq!(total, 30);
        assert_eq!(
            cpu_delta(Some(10), Some(20_000_010), 2.0, 4),
            (Some(2.0), Some(25.0))
        );
        assert_eq!(cpu_delta(None, Some(3), 1.0, 4), (None, None));
        assert_eq!(cpu_delta(Some(4), Some(3), 1.0, 4), (None, None));
        assert_eq!(cpu_delta(Some(0), Some(0), 1.0, 4), (Some(0.0), Some(0.0)));
        assert_eq!(cpu_delta(Some(0), Some(10), 0.0, 4), (None, None));
    }
    #[test]
    fn closed_context_never_copies_external_phase_or_channel() {
        assert_eq!(phase("private-path-and-token"), "unknown");
        assert!(!managed_stem("../anything"));
        assert!(managed_stem(&format!("cmperf-{}", "A".repeat(64))));
        let mut m = Metrics::empty();
        m.add(Stage::Write as usize, 7);
        m.add(Stage::Write as usize, 11);
        assert_eq!(m.stages[7].count, 2);
        assert_eq!(m.stages[7].max_micros, 11);
        assert!(m.valid());
    }
    #[test]
    fn budget_lock_and_unknown_records_preserve_data() {
        let root = root("performance-budget");
        fs::create_dir_all(root.join("logs")).unwrap();
        fs::write(root.join("logs/keep-user.txt"), b"keep").unwrap();
        let f = Files::prepare(&root).unwrap();
        assert!(Files::prepare(&root).is_err());
        drop(f);
        for _ in 0..4 {
            drop(Files::prepare(&root).unwrap());
        }
        assert!(Files::prepare(&root).is_err());
        assert_eq!(fs::read(root.join("logs/keep-user.txt")).unwrap(), b"keep");
    }
    #[test]
    fn logger_fails_independently_and_does_not_fallback() {
        let root = root("performance-blocked");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("logs"), b"blocked").unwrap();
        assert!(Files::prepare(&root).is_err());
        assert_eq!(fs::read(root.join("logs")).unwrap(), b"blocked");
    }
    #[test]
    fn sixty_second_boundary_and_stop_are_independent_reasons() {
        assert_eq!(end_reason(59_999, LIMIT_MS, false, false), None);
        assert_eq!(end_reason(60_000, LIMIT_MS, false, false), Some("duration"));
        assert_eq!(end_reason(10, LIMIT_MS, true, false), Some("manual"));
        assert_eq!(end_reason(10, LIMIT_MS, true, true), Some("exit"));
    }
    fn write_complete(files: &mut Files) {
        files
            .line(&Header {
                kind: "header".into(),
                magic: MAGIC.into(),
                schema: 2,
                sequence: files.sequence,
                version: "fixture".into(),
                logical_processors: 1,
                limit_ms: LIMIT_MS,
                webview: "unavailable".into(),
                limitations: vec![],
            })
            .unwrap();
        files
            .line(&serde_json::json!({"kind":"summary","magic":MAGIC,"complete":true}))
            .unwrap();
        files.flush().unwrap();
        files.text.write_all(MAGIC.as_bytes()).unwrap();
    }
    #[test]
    fn rotation_removes_only_completed_pairs_and_bounds_output() {
        let root = root("performance-rotation");
        let mut oldest = None;
        for _ in 0..KEEP {
            let mut f = Files::prepare(&root).unwrap();
            if oldest.is_none() {
                oldest = Some(f.stem.clone());
            }
            write_complete(&mut f);
        }
        let mut new = Files::prepare(&root).unwrap();
        assert!(!root
            .join("logs")
            .join(format!("{}.jsonl", oldest.unwrap()))
            .exists());
        assert!(new.line(&"x".repeat(SESSION_BYTES)).is_err());
        assert!(new.buffer.is_empty());
        write_complete(&mut new);
        drop(new);
        let files = fs::read_dir(root.join("logs"))
            .unwrap()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        assert_eq!(
            files
                .iter()
                .filter(|f| f.path().extension().is_some_and(|e| e == "jsonl"))
                .count(),
            KEEP
        );
        assert!(
            files
                .iter()
                .map(|f| f.metadata().unwrap().len())
                .sum::<u64>()
                <= (KEEP * SESSION_BYTES) as u64
        );
    }
    #[test]
    fn retained_original_child_handle_accounts_cpu_once_and_clips_start() {
        use std::os::windows::process::CommandExt;
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "performance::tests::fixture_process",
                "--ignored",
                "--test-threads=1",
            ])
            .env("CLICKMASTER_PERF_FIXTURE", "cpu")
            .creation_flags(0x08000000)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let handle = native::duplicate(&child).unwrap();
        let base = native::cpu(handle.0);
        let mut worker = Worker {
            number: 1,
            handle,
            base,
            last: base,
            observed: Instant::now(),
            clipped_start: true,
            collected: false,
        };
        child.wait().unwrap();
        let mut totals = Workers::default();
        let record = collect_worker(&mut worker, &mut totals, true).unwrap();
        assert!(record.clipped_start);
        assert!(!record.clipped_end);
        assert!(record.cpu_seconds.is_some());
        assert_eq!(totals.exited, 1);
        let ticks = totals.cpu_ticks;
        assert!(collect_worker(&mut worker, &mut totals, true).is_none());
        assert_eq!(totals.cpu_ticks, ticks);
    }
    #[test]
    fn capture_end_clips_later_worker_cpu_without_terminating_child() {
        use std::os::windows::process::CommandExt;
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "performance::tests::fixture_process",
                "--ignored",
                "--test-threads=1",
            ])
            .env("CLICKMASTER_PERF_FIXTURE", "hold")
            .creation_flags(0x08000000)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let handle = native::duplicate(&child).unwrap();
        let mut worker = Worker {
            number: 1,
            handle,
            base: Some(0),
            last: Some(0),
            observed: Instant::now(),
            clipped_start: false,
            collected: false,
        };
        let mut totals = Workers::default();
        let record = collect_worker(&mut worker, &mut totals, true).unwrap();
        assert!(record.clipped_end);
        assert!(child.try_wait().unwrap().is_none());
        let ticks = totals.cpu_ticks;
        child.wait().unwrap();
        assert!(collect_worker(&mut worker, &mut totals, true).is_none());
        assert_eq!(totals.cpu_ticks, ticks);
    }
    fn wait_for(mut condition: impl FnMut() -> bool) {
        let start = Instant::now();
        while !condition() {
            assert!(
                start.elapsed() < Duration::from_secs(8),
                "fixture did not reach bounded condition"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }
    #[test]
    fn isolated_logger_lifecycle_and_equal_work_pipe_comparison() {
        use std::{
            os::windows::process::CommandExt,
            process::{Command, Stdio},
        };
        let root = root("performance-integration");
        fs::create_dir_all(&root).unwrap();
        for mode in ["off", "on", "lifecycle", "states"] {
            let result = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "performance::tests::fixture_process",
                    "--ignored",
                    "--test-threads=1",
                ])
                .env("CLICKMASTER_PERF_FIXTURE", mode)
                .env("CLICKMASTER_PERF_ROOT", root.join(mode))
                .creation_flags(0x08000000)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            assert!(
                result.success(),
                "isolated performance fixture failed: {mode}"
            );
        }
        let load = |name: &str| {
            serde_json::from_slice::<serde_json::Value>(
                &fs::read(root.join(name).join("result.json")).unwrap(),
            )
            .unwrap()
        };
        let before = load("off");
        let after = load("on");
        assert_eq!(before["frames"], after["frames"]);
        assert_eq!(after["frames"], 24);
        assert_eq!(after["errors"], 0);
        fs::write(root.join("comparison.json"),serde_json::to_vec_pretty(&serde_json::json!({"before":before,"after":after,"limits":"single sequential sample; non-game private pipe; real collector/native/hash/owned workers; no game identity or WebView/private auth/UI; no zero-overhead claim"})).unwrap()).unwrap();
    }
    #[test]
    #[ignore = "isolated non-game diagnostic subprocess only"]
    fn fixture_process() {
        use std::{
            io::Read,
            os::windows::process::CommandExt,
            process::{Command, Stdio},
        };
        match std::env::var("CLICKMASTER_PERF_FIXTURE").unwrap().as_str() {
            "hold" => thread::sleep(Duration::from_secs(1)),
            "cpu" => {
                let at = Instant::now();
                let mut x = 1u64;
                while at.elapsed() < Duration::from_millis(80) {
                    for _ in 0..10000 {
                        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
                    }
                    std::hint::black_box(x);
                }
            }
            "server" => {
                let epoch = std::env::var("CLICKMASTER_PERF_EPOCH")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap();
                set_worker(epoch != 0);
                {
                    let _span = span(Stage::WorkerExeHash);
                    crate::ipc_native::file_sha(&std::env::current_exe().unwrap()).unwrap();
                }
                let root = PathBuf::from(std::env::var_os("CLICKMASTER_PERF_ROOT").unwrap());
                let pipe =
                    crate::ipc_native::pipe(&std::env::var("CLICKMASTER_PERF_PIPE").unwrap())
                        .unwrap();
                fs::write(root.join("ready"), b"ready").unwrap();
                crate::ipc_native::connect_bounded(&pipe, 2000).unwrap();
                crate::ipc_worker::send_frame(&pipe, || Ok(()), |_, _| Ok(())).unwrap();
                let mut report = ipc::Report::new("completed");
                report.performance = worker_metrics();
                report.performance_epoch = epoch;
                fs::write(root.join("report"), serde_json::to_vec(&report).unwrap()).unwrap();
            }
            "lifecycle" => {
                let root = PathBuf::from(std::env::var_os("CLICKMASTER_PERF_ROOT").unwrap());
                fs::create_dir_all(&root).unwrap();
                assert!(!active());
                assert!(!root.join("logs").exists());
                stop();
                start(root.clone(), crate::input::Status::default(), None).unwrap();
                wait_for(active);
                assert!(start(root.clone(), crate::input::Status::default(), None).is_err());
                window_state(Some(false), Some(false));
                window_state_epoch(Some(true), Some(false), Some(epoch().wrapping_add(1)));
                assert_eq!(global().live.lock().unwrap().window, "tray-hidden");
                let mut initial = crate::input::Status::default();
                initial.phase = "private-string".into();
                initial.channel = "private-channel".into();
                context(&initial);
                thread::sleep(Duration::from_millis(1050));
                stop();
                stop();
                wait_for(|| !global().busy.load(Ordering::Acquire));
                assert!(view().complete);
                let stem = view().file_stem.unwrap();
                let bytes = fs::read(root.join("logs").join(format!("{stem}.jsonl"))).unwrap();
                let text = String::from_utf8(bytes).unwrap();
                assert!(!text.contains("private-string"));
                assert!(!text.contains("private-channel"));
                assert!(text.contains("tray-hidden"));
                assert!(text.contains("unavailable"));
                assert!(text.contains("\"reason\":\"manual\""));
                // The same production loop with a shortened test budget verifies auto completion.
                let c = global().clone();
                c.stop.store(false, Ordering::Relaxed);
                c.exiting.store(false, Ordering::Relaxed);
                *c.live.lock().unwrap() = Live::default();
                capture(&c, &root, 20, &crate::input::Status::default(), None, None).unwrap();
                assert!(view().complete);
                start(root.clone(), crate::input::Status::default(), None).unwrap();
                wait_for(active);
                shutdown();
                assert!(!active());
                // Simulate an unresponsive writer: shutdown remains bounded and never waits on its IO.
                global().busy.store(true, Ordering::Release);
                let at = Instant::now();
                shutdown();
                assert!(at.elapsed() < Duration::from_millis(500));
                global().busy.store(false, Ordering::Release);
                fs::write(root.join("result.json"), b"{\"pass\":true}").unwrap();
            }
            "states" => {
                // Backend event facts only. No native UI, focus, hotkey, input, or COM is run.
                let root = PathBuf::from(std::env::var_os("CLICKMASTER_PERF_ROOT").unwrap());
                fs::create_dir_all(&root).unwrap();
                let mut status = crate::input::Status::default();
                start(root.clone(), status.clone(), None).unwrap();
                wait_for(active);
                for (visible, busy) in [(true, false), (false, false), (false, true), (true, true)]
                {
                    status.busy = busy;
                    context(&status);
                    window_state(Some(visible), Some(false));
                    thread::sleep(Duration::from_millis(2200));
                }
                stop();
                wait_for(|| !global().busy.load(Ordering::Acquire));
                let stem = view().file_stem.unwrap();
                let text =
                    fs::read_to_string(root.join("logs").join(format!("{stem}.jsonl"))).unwrap();
                let summary: serde_json::Value =
                    serde_json::from_str(text.lines().last().unwrap()).unwrap();
                let bins = summary["stateComparison"]["bins"].as_array().unwrap();
                for bin in bins.iter().take(4) {
                    assert_eq!(bin["availability"], "observed");
                }
                assert!(bins[4]["intervals"].as_u64().unwrap() >= 1);
                fs::write(
                    root.join("result.json"),
                    b"{\"pass\":true,\"realUiOrCom\":false}",
                )
                .unwrap();
            }
            mode @ ("on" | "off") => {
                let enabled = mode == "on";
                let root = PathBuf::from(std::env::var_os("CLICKMASTER_PERF_ROOT").unwrap());
                fs::create_dir_all(&root).unwrap();
                let at = Instant::now();
                let cpu_start = native::current_cpu().unwrap();
                let mut worker_cpu = 0u64;
                let mut status = crate::input::Status::default();
                status.channel = "taptaploot-ipc".into();
                status.phase = "waiting".into();
                status.task_id = 1;
                status.repeating = true;
                if enabled {
                    start(
                        root.clone(),
                        status.clone(),
                        Some(crate::repeat::Range { min: 100, max: 100 }),
                    )
                    .unwrap();
                    wait_for(active);
                }
                let mut slot = crate::ipc_storage::Slot::new(&root);
                for index in 0..24 {
                    let (dir, _) = slot.prepare().unwrap();
                    let name = format!(
                        r"\\.\pipe\ClickMaster-OfflineFixture-{}",
                        crate::ipc_native::random().unwrap()
                    );
                    let mut child = {
                        let _span = span(Stage::Spawn);
                        Command::new(std::env::current_exe().unwrap())
                            .args([
                                "--exact",
                                "performance::tests::fixture_process",
                                "--ignored",
                                "--test-threads=1",
                            ])
                            .env("CLICKMASTER_PERF_FIXTURE", "server")
                            .env("CLICKMASTER_PERF_ROOT", &dir)
                            .env("CLICKMASTER_PERF_PIPE", name.clone())
                            .env("CLICKMASTER_PERF_EPOCH", epoch().to_string())
                            .creation_flags(0x08000000)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()
                            .unwrap()
                    };
                    observe_child(&child, true);
                    wait_for(|| dir.join("ready").exists());
                    {
                        let mut pipe = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(name)
                            .unwrap();
                        let mut bytes = [0u8; 4];
                        pipe.read_exact(&mut bytes).unwrap();
                        assert_eq!(bytes, ipc::FRAME);
                    }
                    while child.try_wait().unwrap().is_none() {
                        observe_child(&child, false);
                        thread::sleep(Duration::from_millis(2));
                    }
                    worker_cpu += native::cpu(child.as_raw_handle()).unwrap();
                    finish_child(&child);
                    let report: ipc::Report =
                        serde_json::from_slice(&fs::read(dir.join("report")).unwrap()).unwrap();
                    merge_worker(report.performance.as_ref(), false, report.performance_epoch);
                    fs::remove_file(dir.join("ready")).unwrap();
                    fs::remove_file(dir.join("report")).unwrap();
                    status.completed = index + 1;
                    context(&status);
                    thread::sleep(Duration::from_millis(100));
                }
                slot.finish().unwrap();
                thread::sleep(Duration::from_millis(3000));
                if enabled {
                    // Exercise the real five-second batch flush before the final stop.
                    let stem = view().file_stem.unwrap();
                    assert!(
                        fs::metadata(root.join("logs").join(format!("{stem}.jsonl")))
                            .unwrap()
                            .len()
                            > 0
                    );
                    stop();
                    wait_for(|| !global().busy.load(Ordering::Acquire));
                    let v = view();
                    assert!(v.complete);
                    let b = fs::read(
                        root.join("logs")
                            .join(format!("{}.jsonl", v.file_stem.unwrap())),
                    )
                    .unwrap();
                    let rows = b
                        .split(|c| *c == b'\n')
                        .filter(|s| !s.is_empty())
                        .map(|s| serde_json::from_slice::<serde_json::Value>(s).unwrap())
                        .collect::<Vec<_>>();
                    let summary = rows.last().unwrap();
                    assert_eq!(summary["workers"]["observed"], 24);
                    assert_eq!(summary["workers"]["exited"], 24);
                    assert_eq!(summary["sentObserved"], 24);
                    assert_eq!(summary["workers"]["cpuMissing"], 0);
                    assert!(b.len() < SESSION_BYTES);
                } else {
                    assert!(!root.join("logs").exists());
                }
                let cpu = native::current_cpu().unwrap() - cpu_start;
                let seconds = at.elapsed().as_secs_f64();
                fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"enabled":enabled,"frames":24,"workers":24,"errors":0,"elapsedSeconds":seconds,"senderCpuSeconds":cpu as f64/10_000_000.0,"workerCpuSeconds":worker_cpu as f64/10_000_000.0,"combinedCpuSeconds":(cpu+worker_cpu) as f64/10_000_000.0,"logicalProcessors":thread::available_parallelism().unwrap().get()})).unwrap()).unwrap();
            }
            _ => panic!("unknown fixture"),
        }
    }
}
