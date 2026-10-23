//! Fixed-size interval grouping. Transitions inside an interval cannot be assigned to its endpoint.
use crate::perf_webview;
use serde::Serialize;

pub fn window(visible: Option<bool>, minimized: Option<bool>) -> &'static str {
    match (visible, minimized) {
        (Some(false), _) => "tray-hidden",
        (Some(true), Some(true)) => "minimized",
        (Some(true), Some(false)) => "visible",
        _ => "unknown",
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub running: Option<bool>,
    pub window: &'static str,
    pub revision: u64,
    pub lost: u64,
}
pub fn group(start: State, end: State) -> usize {
    if start.revision != end.revision || start.lost != end.lost {
        return 4;
    }
    match (end.running, end.window) {
        (Some(false), "visible") => 0,
        (Some(false), "tray-hidden" | "minimized") => 1,
        (Some(true), "visible") => 2,
        (Some(true), "tray-hidden" | "minimized") => 3,
        _ => 5,
    }
}
const NAMES: [&str; 6] = [
    "idle-visible",
    "idle-hidden",
    "running-visible",
    "running-hidden",
    "mixed",
    "unknown",
];
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cpu {
    seconds_observed: Option<f64>,
    interval_seconds_observed: Option<f64>,
    present_intervals: u64,
    missing_intervals: u64,
    partial_intervals: u64,
    machine_cpu_percent_observed: Option<f64>,
}
impl Cpu {
    fn add(&mut self, value: Option<f64>, seconds: f64, partial: bool, processors: usize) {
        if let Some(value) = value {
            *self.seconds_observed.get_or_insert(0.0) += value;
            *self.interval_seconds_observed.get_or_insert(0.0) += seconds;
            self.present_intervals += 1;
            self.partial_intervals += u64::from(partial);
            self.machine_cpu_percent_observed = self
                .seconds_observed
                .zip(self.interval_seconds_observed)
                .filter(|(_, s)| *s > 0.0 && processors > 0)
                .map(|(c, s)| c / s / processors as f64 * 100.0);
        } else {
            self.missing_intervals += 1;
        }
    }
}
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    samples: u64,
    missing_samples: u64,
    mean_bytes_observed: Option<f64>,
    max_bytes_observed: Option<u64>,
    #[serde(skip)]
    total: f64,
}
impl Memory {
    fn add(&mut self, value: Option<u64>) {
        if let Some(value) = value {
            self.samples += 1;
            self.total += value as f64;
            self.max_bytes_observed = Some(self.max_bytes_observed.unwrap_or(0).max(value));
            self.mean_bytes_observed = Some(self.total / self.samples as f64);
        } else {
            self.missing_samples += 1;
        }
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bin {
    name: &'static str,
    availability: &'static str,
    intervals: u64,
    interval_seconds: Option<f64>,
    sent_observed: Option<u64>,
    sends_per_second: Option<f64>,
    main_cpu: Cpu,
    worker_cpu: Cpu,
    webview_environment_cpu: Cpu,
    main_working_set: Memory,
    main_private: Memory,
    webview_environment_working_set: Memory,
    webview_environment_private: Memory,
    webview_fresh_intervals: u64,
    webview_partial_intervals: u64,
    webview_unavailable_intervals: u64,
}
impl Bin {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            availability: "not-observed",
            intervals: 0,
            interval_seconds: None,
            sent_observed: None,
            sends_per_second: None,
            main_cpu: Cpu::default(),
            worker_cpu: Cpu::default(),
            webview_environment_cpu: Cpu::default(),
            main_working_set: Memory::default(),
            main_private: Memory::default(),
            webview_environment_working_set: Memory::default(),
            webview_environment_private: Memory::default(),
            webview_fresh_intervals: 0,
            webview_partial_intervals: 0,
            webview_unavailable_intervals: 0,
        }
    }
}
pub struct Comparison {
    bins: [Bin; 6],
}
impl Default for Comparison {
    fn default() -> Self {
        Self {
            bins: NAMES.map(Bin::new),
        }
    }
}
impl Comparison {
    pub fn add(
        &mut self,
        index: usize,
        seconds: f64,
        sent: u64,
        main: Option<f64>,
        workers: Option<f64>,
        memory: Option<(u64, u64)>,
        webview: &perf_webview::Sample,
        processors: usize,
    ) {
        let bin = &mut self.bins[index];
        bin.availability = "observed";
        bin.intervals += 1;
        *bin.interval_seconds.get_or_insert(0.0) += seconds;
        *bin.sent_observed.get_or_insert(0) += sent;
        bin.sends_per_second = bin
            .sent_observed
            .zip(bin.interval_seconds)
            .map(|(n, s)| n as f64 / s);
        bin.main_cpu.add(main, seconds, false, processors);
        bin.worker_cpu.add(workers, seconds, false, processors);
        bin.webview_environment_cpu.add(
            webview.cpu_seconds_observed,
            seconds,
            webview.cpu_coverage != "complete-observed-interval",
            processors,
        );
        bin.main_working_set.add(memory.map(|m| m.0));
        bin.main_private.add(memory.map(|m| m.1));
        bin.webview_environment_working_set
            .add(webview.working_set_bytes_observed);
        bin.webview_environment_private
            .add(webview.private_bytes_observed);
        if webview.freshness == "fresh" {
            bin.webview_fresh_intervals += 1;
        }
        if webview.availability == "partial" {
            bin.webview_partial_intervals += 1;
        }
        if webview.availability == "unavailable" {
            bin.webview_unavailable_intervals += 1;
        }
    }
    pub fn name(index: usize) -> &'static str {
        NAMES[index]
    }
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({"bins":self.bins,"coverage":"sample intervals only; capture tail excluded; mixed and unknown excluded from strict four-state comparison",
            "cpuDenominator":"each component uses its observed interval duration; partial components are lower-bound observations, not full totals",
            "webviewScope":"webview-environment; sharedPossible=true; excludes crashpad",
            "memory":"one-second snapshots; working-set sums may overlap/shared; not exclusive physical memory"})
    }
    pub fn text(&self) -> String {
        let mut text =
            String::from("\n四状态对照（仅采样区间，采集尾段不分组；mixed/unknown不归入四组）：\n");
        for bin in &self.bins {
            text.push_str(&format!("{}：{}；时长{:?}s；发送{:?}；CPU秒 主{:?}/worker{:?}/WebView环境{:?}；WebView新鲜/部分/缺失={}/{}/{}\n",
                bin.name, bin.availability, bin.interval_seconds, bin.sent_observed,
                bin.main_cpu.seconds_observed, bin.worker_cpu.seconds_observed, bin.webview_environment_cpu.seconds_observed,
                bin.webview_fresh_intervals, bin.webview_partial_intervals, bin.webview_unavailable_intervals));
            text.push_str(&format!(
                "  内存工作集均值/样本：主{:?}B/{}，WebView环境{:?}B/{}；共享工作集可能重叠。\n",
                bin.main_working_set.mean_bytes_observed,
                bin.main_working_set.samples,
                bin.webview_environment_working_set.mean_bytes_observed,
                bin.webview_environment_working_set.samples
            ));
        }
        text
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transitions_roundtrips_losses_and_unknowns_never_use_endpoint_state() {
        let first = State {
            running: Some(false),
            window: "visible",
            revision: 0,
            lost: 0,
        };
        assert_eq!(group(first, first), 0);
        let hidden = State {
            running: Some(true),
            window: "tray-hidden",
            revision: 2,
            lost: 0,
        };
        assert_eq!(group(hidden, hidden), 3);
        assert_eq!(group(first, hidden), 4);
        assert_eq!(
            group(
                first,
                State {
                    revision: 2,
                    ..first
                }
            ),
            4
        ); // away and back
        assert_eq!(group(first, State { lost: 1, ..first }), 4);
        let unknown = State {
            running: None,
            ..first
        };
        assert_eq!(group(unknown, unknown), 5);
        let mut comparison = Comparison::default();
        let unavailable = perf_webview::Sampler::default().sample(None, 1.0, 4);
        comparison.add(0, 2.0, 4, Some(1.0), None, Some((100, 50)), &unavailable, 4);
        let bins = comparison.json()["bins"].clone();
        assert_eq!(bins[0]["mainCpu"]["machineCpuPercentObserved"], 12.5);
        assert_eq!(bins[0]["workerCpu"]["missingIntervals"], 1);
        assert!(bins[0]["webviewEnvironmentCpu"]["secondsObserved"].is_null());
        assert_eq!(bins[3]["availability"], "not-observed");
        assert!(bins[3]["sentObserved"].is_null());
        assert_eq!(window(Some(false), None), "tray-hidden");
        assert_eq!(window(Some(true), None), "unknown");
    }
}
