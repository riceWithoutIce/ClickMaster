//! Optional UI-thread COM adapter and bounded, anonymous environment resource sampling.
//! The environment can be shared; these are never EXE-exclusive resource totals.
use crate::{ipc_native::Handle, perf_native as native};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use windows_core::Interface;

pub const MAX_PROCESSES: usize = 64;
const FRESH: Duration = Duration::from_millis(2500);
static UI_PENDING: OnceLock<Arc<AtomicBool>> = OnceLock::new();

struct Permit(Arc<AtomicBool>);
impl Permit {
    fn acquire(gate: &Arc<AtomicBool>) -> Option<Self> {
        gate.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .ok()
            .map(|_| Self(gate.clone()))
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
pub struct Process {
    pid: u32,
    stamp: u64,
    kind: &'static str,
    handle: Handle,
}
struct Reply {
    at: Instant,
    reason: &'static str,
    listed: usize,
    missing: usize,
    truncated: bool,
    processes: Vec<Process>,
}
impl Reply {
    fn unavailable(reason: &'static str) -> Self {
        Self {
            at: Instant::now(),
            reason,
            listed: 0,
            missing: 0,
            truncated: false,
            processes: Vec::new(),
        }
    }
}
struct Shared {
    alive: AtomicBool,
    reply: Mutex<Option<Reply>>,
}
/// One per capture: an old UI callback cannot write into a later capture's slot.
pub struct Source {
    window: tauri::WebviewWindow,
    shared: Arc<Shared>,
    pub initial_window: (Option<bool>, Option<bool>),
}
impl Source {
    pub fn new(window: tauri::WebviewWindow) -> Self {
        let initial_window = (window.is_visible().ok(), window.is_minimized().ok());
        Self {
            window,
            initial_window,
            shared: Arc::new(Shared {
                alive: AtomicBool::new(true),
                reply: Mutex::new(None),
            }),
        }
    }
    pub fn request(&self, epoch: u64) {
        if !self.shared.alive.load(Ordering::Acquire) {
            return;
        }
        let gate = UI_PENDING.get_or_init(|| Arc::new(AtomicBool::new(false)));
        let Some(permit) = Permit::acquire(gate) else {
            return;
        };
        let shared = self.shared.clone();
        let window = self.window.clone();
        let queued = Instant::now();
        // with_webview schedules this closure on the UI thread. No COM object leaves it.
        if self
            .window
            .with_webview(move |webview| {
                let _permit = permit;
                if !shared.alive.load(Ordering::Acquire) || queued.elapsed() > FRESH {
                    return;
                }
                crate::performance::window_state_epoch(
                    window.is_visible().ok(),
                    window.is_minimized().ok(),
                    Some(epoch),
                );
                let reply = unsafe { enumerate(&webview) };
                if shared.alive.load(Ordering::Acquire) && queued.elapsed() <= FRESH {
                    if let Ok(mut slot) = shared.reply.try_lock() {
                        *slot = Some(reply);
                    }
                }
            })
            .is_err()
        {
            if let Ok(mut slot) = self.shared.reply.try_lock() {
                *slot = Some(Reply::unavailable("ui-dispatch-unavailable"));
            }
        }
    }
    fn take(&self) -> Option<Reply> {
        self.shared.reply.try_lock().ok()?.take()
    }
    pub fn cancel(&self) {
        self.shared.alive.store(false, Ordering::Release);
    }
}
impl Drop for Source {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn kind(value: COREWEBVIEW2_PROCESS_KIND) -> &'static str {
    match value {
        COREWEBVIEW2_PROCESS_KIND_BROWSER => "browser",
        COREWEBVIEW2_PROCESS_KIND_RENDERER => "renderer",
        COREWEBVIEW2_PROCESS_KIND_UTILITY => "utility",
        COREWEBVIEW2_PROCESS_KIND_SANDBOX_HELPER => "sandbox-helper",
        COREWEBVIEW2_PROCESS_KIND_GPU => "gpu",
        COREWEBVIEW2_PROCESS_KIND_PPAPI_PLUGIN => "ppapi-plugin",
        COREWEBVIEW2_PROCESS_KIND_PPAPI_BROKER => "ppapi-broker",
        _ => "unknown",
    }
}
unsafe fn enumerate(webview: &tauri::webview::PlatformWebview) -> Reply {
    let Ok(env) = webview.environment().cast::<ICoreWebView2Environment8>() else {
        return Reply::unavailable("environment8-unsupported");
    };
    let Ok(list) = env.GetProcessInfos() else {
        return Reply::unavailable("enumeration-failed");
    };
    let mut count = 0;
    if list.Count(&mut count).is_err() {
        return Reply::unavailable("enumeration-failed");
    }
    let mut reply = Reply {
        at: Instant::now(),
        reason: "official-environment-list",
        listed: count as usize,
        missing: 0,
        truncated: count as usize > MAX_PROCESSES,
        processes: Vec::new(),
    };
    let mut seen = BTreeSet::new();
    for index in 0..count.min(MAX_PROCESSES as u32) {
        let Ok(info) = list.GetValueAtIndex(index) else {
            reply.missing += 1;
            continue;
        };
        let mut pid = 0;
        let mut value = COREWEBVIEW2_PROCESS_KIND(-1);
        if info.ProcessId(&mut pid).is_err() || pid <= 0 {
            reply.missing += 1;
            continue;
        }
        if !seen.insert(pid) {
            continue;
        }
        // Unknown type is represented explicitly, without counting one missing process twice.
        let _ = info.Kind(&mut value);
        let Some((handle, stamp)) = native::open_process(pid as u32) else {
            reply.missing += 1;
            continue;
        };
        reply.processes.push(Process {
            pid: pid as u32,
            stamp,
            kind: kind(value),
            handle,
        });
    }
    reply.at = Instant::now();
    reply
}

struct Tracked {
    process: Process,
    last: Option<u64>,
    ended: bool,
}
#[derive(Default)]
pub struct Sampler {
    tracked: BTreeMap<(u32, u64), Tracked>,
    at: Option<Instant>,
    reason: Option<&'static str>,
    listed: usize,
    missing: usize,
    truncated: bool,
    previous_fresh: bool,
    waiting_since: Option<Instant>,
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub scope: &'static str,
    pub shared_possible: bool,
    pub excludes_crashpad: bool,
    pub availability: &'static str,
    pub reason: &'static str,
    pub freshness: &'static str,
    pub age_ms: Option<u64>,
    pub listed_processes: Option<usize>,
    pub opened_processes: usize,
    pub kinds: BTreeMap<&'static str, usize>,
    pub missing_processes: usize,
    pub truncated: bool,
    pub membership_changed: bool,
    pub exited_processes: usize,
    pub cpu_seconds_observed: Option<f64>,
    pub machine_cpu_percent_observed: Option<f64>,
    pub cpu_coverage: &'static str,
    pub cpu_missing_processes: usize,
    pub working_set_bytes_observed: Option<u64>,
    pub private_bytes_observed: Option<u64>,
    pub memory_missing_processes: usize,
    pub memory_note: &'static str,
}
impl Sample {
    fn missing(reason: &'static str, age_ms: Option<u64>) -> Self {
        Self {
            scope: "webview-environment",
            shared_possible: true,
            excludes_crashpad: true,
            availability: "unavailable",
            reason,
            freshness: if age_ms.is_some() {
                "stale"
            } else {
                "not-observed"
            },
            age_ms,
            listed_processes: None,
            opened_processes: 0,
            kinds: BTreeMap::new(),
            missing_processes: 0,
            truncated: false,
            membership_changed: false,
            exited_processes: 0,
            cpu_seconds_observed: None,
            machine_cpu_percent_observed: None,
            cpu_coverage: "unavailable",
            cpu_missing_processes: 0,
            working_set_bytes_observed: None,
            private_bytes_observed: None,
            memory_missing_processes: 0,
            memory_note: "working-set sums can overlap/shared; not exclusive physical memory",
        }
    }
}
impl Sampler {
    pub fn sample(&mut self, source: Option<&Source>, seconds: f64, processors: usize) -> Sample {
        let reply = source.and_then(Source::take);
        self.advance(reply, source.is_some(), seconds, processors, Instant::now())
    }
    fn advance(
        &mut self,
        reply: Option<Reply>,
        enabled: bool,
        seconds: f64,
        processors: usize,
        now: Instant,
    ) -> Sample {
        let waiting_since = *self.waiting_since.get_or_insert(now);
        let mut changed = false;
        let mut removed_missing = 0;
        let mut exited = 0;
        let mut terminal_cpu = None::<u64>;
        if let Some(reply) = reply {
            self.at = Some(reply.at);
            self.reason = Some(reply.reason);
            self.listed = reply.listed;
            self.missing = reply.missing;
            self.truncated = reply.truncated;
            let mut previous = std::mem::take(&mut self.tracked);
            for process in reply.processes {
                let key = (process.pid, process.stamp);
                if self.tracked.contains_key(&key) {
                    continue;
                }
                let tracked = if let Some(old) = previous.remove(&key) {
                    old
                } else {
                    changed = true;
                    Tracked {
                        process,
                        last: None,
                        ended: false,
                    }
                };
                self.tracked.insert(key, tracked);
            }
            for old in previous.into_values() {
                changed = true;
                if old.ended {
                    continue;
                }
                if native::exited(&old.process.handle) && self.previous_fresh {
                    exited += 1;
                    if let Some(delta) = ticks(old.last, native::cpu(old.process.handle.0)) {
                        *terminal_cpu.get_or_insert(0) += delta;
                    } else {
                        removed_missing += 1;
                    }
                } else {
                    removed_missing += 1;
                }
            }
        }
        let age = self.at.map(|t| now.saturating_duration_since(t));
        if !enabled
            || age.is_none_or(|a| a > FRESH)
            || self.reason != Some("official-environment-list")
            || (self.tracked.is_empty() && terminal_cpu.is_none())
        {
            self.previous_fresh = false;
            for p in self.tracked.values_mut() {
                p.last = None;
            }
            let reason = if !enabled {
                "source-unavailable"
            } else if age.is_none() {
                if now.saturating_duration_since(waiting_since) > FRESH {
                    "ui-query-timeout"
                } else {
                    "ui-query-pending"
                }
            } else if age.is_some_and(|a| a > FRESH) {
                "ui-query-stale-or-timeout"
            } else {
                if self.reason == Some("official-environment-list") {
                    "empty-environment-list"
                } else {
                    self.reason.unwrap_or("source-unavailable")
                }
            };
            let mut missing = Sample::missing(reason, age.map(|a| a.as_millis() as u64));
            missing.freshness = if !enabled {
                "not-observed"
            } else if age.is_none() {
                if now.saturating_duration_since(waiting_since) > FRESH {
                    "timeout"
                } else {
                    "pending"
                }
            } else if age.is_some_and(|a| a <= FRESH) {
                "fresh-query-unavailable"
            } else {
                "stale"
            };
            missing.listed_processes = self.at.map(|_| self.listed);
            missing.missing_processes = self.missing + removed_missing;
            missing.truncated = self.truncated;
            missing.membership_changed = changed;
            missing.exited_processes = exited;
            return missing;
        }
        let mut out = Sample::missing(
            "official-environment-list",
            age.map(|a| a.as_millis() as u64),
        );
        out.availability = "available";
        out.freshness = "fresh";
        out.listed_processes = Some(self.listed);
        out.opened_processes = self.tracked.len();
        out.missing_processes = self.missing;
        out.truncated = self.truncated;
        out.membership_changed = changed;
        out.exited_processes = exited;
        let mut cpu = terminal_cpu;
        out.cpu_missing_processes = removed_missing;
        for tracked in self.tracked.values_mut() {
            *out.kinds.entry(tracked.process.kind).or_default() += 1;
            let raw = native::cpu(tracked.process.handle.0);
            if let Some(delta) = ticks(tracked.last, raw).filter(|_| self.previous_fresh) {
                *cpu.get_or_insert(0) += delta;
            } else {
                out.cpu_missing_processes += 1;
            }
            tracked.last = raw;
            if native::exited(&tracked.process.handle) {
                if !tracked.ended {
                    out.exited_processes += 1;
                }
                tracked.ended = true;
                out.memory_missing_processes += 1;
            } else if let Some((ws, private)) = native::process_memory(tracked.process.handle.0) {
                *out.working_set_bytes_observed.get_or_insert(0) += ws;
                *out.private_bytes_observed.get_or_insert(0) += private;
            } else {
                out.memory_missing_processes += 1;
            }
        }
        self.previous_fresh = true;
        out.cpu_seconds_observed = cpu.map(|ticks| ticks as f64 / 10_000_000.0);
        out.machine_cpu_percent_observed = out
            .cpu_seconds_observed
            .filter(|_| processors > 0 && seconds > 0.0)
            .map(|c| c / seconds / processors as f64 * 100.0);
        let partial = changed
            || self.missing > 0
            || self.truncated
            || out.exited_processes > 0
            || out.kinds.contains_key("unknown");
        out.cpu_coverage = if cpu.is_none() {
            "unavailable"
        } else if partial || out.cpu_missing_processes > 0 {
            "partial"
        } else {
            "complete-observed-interval"
        };
        if partial || out.cpu_missing_processes > 0 || out.memory_missing_processes > 0 {
            out.availability = "partial";
        }
        out
    }
}
fn ticks(previous: Option<u64>, now: Option<u64>) -> Option<u64> {
    previous
        .zip(now)
        .filter(|(a, b)| b >= a)
        .map(|(a, b)| b - a)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_pending_permit_survives_cancel_and_epoch_restart() {
        let gate = Arc::new(AtomicBool::new(false));
        let first = Permit::acquire(&gate).unwrap();
        assert!(Permit::acquire(&gate).is_none());
        let old = Arc::new(Shared {
            alive: AtomicBool::new(true),
            reply: Mutex::new(None),
        });
        old.alive.store(false, Ordering::Release);
        let new = Arc::new(Shared {
            alive: AtomicBool::new(true),
            reply: Mutex::new(None),
        });
        assert!(Permit::acquire(&gate).is_none()); // timeout is not permission to enqueue a second closure
        assert!(!old.alive.load(Ordering::Acquire));
        assert!(new.reply.lock().unwrap().is_none());
        drop(first);
        assert!(Permit::acquire(&gate).is_some());
    }
    fn own_process(pid: u32, stamp: u64) -> Process {
        let (handle, _) = native::open_process(std::process::id()).unwrap();
        Process {
            pid,
            stamp,
            kind: "renderer",
            handle,
        }
    }
    fn reply(at: Instant, processes: Vec<Process>) -> Reply {
        Reply {
            at,
            listed: processes.len(),
            missing: 0,
            truncated: false,
            reason: "official-environment-list",
            processes,
        }
    }
    #[test]
    fn original_handles_identity_dedup_missing_and_staleness() {
        let mut sampler = Sampler::default();
        let now = Instant::now();
        let first = sampler.advance(
            Some(reply(now, vec![own_process(1, 10), own_process(1, 10)])),
            true,
            1.0,
            4,
            now,
        );
        assert_eq!(first.opened_processes, 1);
        assert!(first.cpu_seconds_observed.is_none()); // never charge process lifetime at first sight
        assert!(first.working_set_bytes_observed.is_some());
        let second = sampler.advance(
            Some(reply(now, vec![own_process(1, 10)])),
            true,
            1.0,
            4,
            now,
        );
        assert!(second.cpu_seconds_observed.is_some());
        assert!(!second.membership_changed);
        assert_eq!(second.cpu_coverage, "complete-observed-interval");
        let reused = sampler.advance(
            Some(reply(now, vec![own_process(1, 11)])),
            true,
            1.0,
            4,
            now,
        );
        assert!(reused.membership_changed);
        assert!(reused.cpu_seconds_observed.is_none());
        let stale = sampler.advance(None, true, 1.0, 4, now + FRESH + Duration::from_millis(1));
        assert_eq!(stale.availability, "unavailable");
        assert!(stale.private_bytes_observed.is_none());
        let fresh = sampler.advance(
            Some(reply(now, vec![own_process(1, 11)])),
            true,
            1.0,
            4,
            now,
        );
        assert!(fresh.cpu_seconds_observed.is_none()); // never spread stale gap over a one-second interval
        let off = Sampler::default().sample(None, 1.0, 4);
        assert_eq!(off.scope, "webview-environment");
        assert!(off.shared_possible && off.excludes_crashpad);
        assert!(off.cpu_seconds_observed.is_none());
        assert_eq!(ticks(Some(10), Some(9)), None);
        assert_eq!(ticks(None, Some(9)), None);
    }
    #[test]
    fn missing_interface_timeout_and_access_denial_are_never_zero() {
        let now = Instant::now();
        let mut sampler = Sampler::default();
        let pending = sampler.advance(None, true, 1.0, 4, now);
        assert_eq!(pending.freshness, "pending");
        let timed = sampler.advance(None, true, 1.0, 4, now + FRESH + Duration::from_millis(1));
        assert_eq!(timed.freshness, "timeout");
        let unsupported = sampler.advance(
            Some(Reply::unavailable("environment8-unsupported")),
            true,
            1.0,
            4,
            now,
        );
        assert_eq!(unsupported.reason, "environment8-unsupported");
        assert!(unsupported.machine_cpu_percent_observed.is_none());
        let mut denied = reply(now, vec![own_process(1, 1)]);
        denied.missing = 1;
        denied.listed = 2;
        denied.truncated = true;
        let partial = sampler.advance(Some(denied), true, 1.0, 4, now);
        assert_eq!(partial.availability, "partial");
        assert_eq!(partial.missing_processes, 1);
        assert_eq!(partial.listed_processes, Some(2));
        assert!(partial.truncated);
        assert_eq!(kind(COREWEBVIEW2_PROCESS_KIND(-100)), "unknown");
    }
    #[test]
    fn exited_original_handle_contributes_final_delta_once_without_reopening_pid() {
        use std::{
            os::windows::process::CommandExt,
            process::{Command, Stdio},
        };
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "performance::tests::fixture_process",
                "--ignored",
                "--test-threads=1",
            ])
            .env("CLICKMASTER_PERF_FIXTURE", "cpu")
            .creation_flags(0x08000000)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let handle = native::duplicate(&child).unwrap();
        let stamp = native::creation(handle.0).unwrap();
        let process = Process {
            pid: child.id(),
            stamp,
            kind: "renderer",
            handle,
        };
        let mut sampler = Sampler::default();
        let now = Instant::now();
        sampler.advance(Some(reply(now, vec![process])), true, 1.0, 4, now);
        let tracked = sampler.tracked.values().next().unwrap();
        let base = tracked.last;
        child.wait().unwrap();
        let expected = ticks(
            base,
            native::cpu(sampler.tracked.values().next().unwrap().process.handle.0),
        );
        let final_sample = sampler.advance(
            Some(reply(Instant::now(), vec![])),
            true,
            1.0,
            4,
            Instant::now(),
        );
        assert_eq!(final_sample.exited_processes, 1);
        assert_eq!(
            final_sample.cpu_seconds_observed,
            expected.map(|t| t as f64 / 10_000_000.0)
        );
        let after = sampler.advance(
            Some(reply(Instant::now(), vec![])),
            true,
            1.0,
            4,
            Instant::now(),
        );
        assert_eq!(after.exited_processes, 0);
        assert!(after.cpu_seconds_observed.is_none());
    }
}
