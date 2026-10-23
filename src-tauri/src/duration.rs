//! Session-only monotonic execution clock. Cleanup is separate from execution time.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Policy {
    #[default]
    Permanent,
    Timed(u64),
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub mode: String,
    pub hours: Option<f64>,
    pub minutes: Option<f64>,
    pub seconds: Option<f64>,
}
impl Request {
    pub fn validate(self) -> Result<Policy, String> {
        if self.mode == "permanent" {
            return Ok(Policy::Permanent);
        }
        if self.mode != "timed" {
            return Err("未知持续时间模式。".into());
        }
        let values = [self.hours, self.minutes, self.seconds];
        for (value, max) in values.iter().zip([99.0, 59.0, 59.0]) {
            if !value.is_some_and(|v| v.is_finite() && v.fract() == 0.0 && v >= 0.0 && v <= max) {
                return Err("限时需填写小时0～99、分钟及秒0～59的整数，不会自动修正。".into());
            }
        }
        let seconds = self.hours.unwrap() as u64 * 3600
            + self.minutes.unwrap() as u64 * 60
            + self.seconds.unwrap() as u64;
        if seconds == 0 {
            return Err("限时至少1秒。".into());
        }
        Ok(Policy::Timed(seconds))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Clock {
    pub limit_seconds: Option<u64>,
    pub elapsed_seconds: u64,
    pub remaining_seconds: Option<u64>,
    pub started: bool,
    pub end_reason: Option<String>,
    pub trigger_reason: Option<String>,
    #[serde(skip)]
    start: Option<u64>,
    #[serde(skip)]
    deadline: Option<u64>,
    #[serde(skip)]
    ended: bool,
}
impl Clock {
    pub fn new(policy: Policy) -> Self {
        let limit = match policy {
            Policy::Permanent => None,
            Policy::Timed(seconds) => Some(seconds),
        };
        Self {
            limit_seconds: limit,
            remaining_seconds: limit,
            ..Self::default()
        }
    }
    pub fn begin(&mut self, now: u64) -> Result<(), String> {
        if self.started || self.ended {
            return Ok(());
        }
        self.deadline = self
            .limit_seconds
            .map(|seconds| {
                seconds
                    .checked_mul(1000)
                    .and_then(|ms| now.checked_add(ms))
                    .ok_or("持续时间超过时钟安全范围。".to_string())
            })
            .transpose()?;
        self.start = Some(now);
        self.started = true;
        Ok(())
    }
    pub fn expired(&self, now: u64) -> bool {
        self.deadline.is_some_and(|end| now >= end)
    }
    pub fn refresh(&mut self, now: u64) {
        if self.ended {
            return;
        }
        if let Some(start) = self.start {
            self.elapsed_seconds = now.saturating_sub(start) / 1000;
            self.remaining_seconds = self
                .deadline
                .map(|end| end.saturating_sub(now).div_ceil(1000));
        }
    }
    pub fn finish(&mut self, now: u64, reason: &str) {
        if self.ended {
            return;
        }
        self.refresh(now);
        self.ended = true;
        if matches!(reason, "时长已到" | "手动停止") {
            self.trigger_reason = Some(reason.into());
        }
        self.end_reason = Some(reason.into());
    }
}
pub fn reason(phase: &str) -> &'static str {
    match phase {
        "expired" => "时长已到",
        "cancelled" => "手动停止",
        "target-invalid" => "目标失效",
        "delivered" | "completed" => "单次已完成",
        "partial" => "部分投递",
        "timeout" => "执行超时",
        "cleanup-failed" => "清理未确认",
        _ => "执行失败",
    }
}
/// Terminal errors take precedence over deadline/manual interruption.
pub fn interruption(phase: &str, stopped: bool, expired: bool) -> Option<&'static str> {
    if matches!(
        phase,
        "failed" | "timeout" | "partial" | "cleanup-failed" | "target-invalid"
    ) {
        return None;
    }
    if stopped {
        Some("cancelled")
    } else if expired {
        Some("expired")
    } else {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validation_modes_and_boundaries() {
        let request = |h, m, s| Request {
            mode: "timed".into(),
            hours: h,
            minutes: m,
            seconds: s,
        };
        assert_eq!(
            request(Some(0.0), Some(0.0), Some(1.0)).validate().unwrap(),
            Policy::Timed(1)
        );
        assert_eq!(
            request(Some(99.0), Some(59.0), Some(59.0))
                .validate()
                .unwrap(),
            Policy::Timed(359999)
        );
        for (h, m, s) in [
            (None, Some(0.0), Some(1.0)),
            (Some(100.0), Some(0.0), Some(0.0)),
            (Some(0.0), Some(60.0), Some(0.0)),
            (Some(0.0), Some(0.0), Some(0.0)),
            (Some(0.0), Some(-1.0), Some(2.0)),
            (Some(0.0), Some(0.0), Some(0.5)),
            (Some(f64::INFINITY), Some(0.0), Some(1.0)),
        ] {
            assert!(request(h, m, s).validate().is_err());
        }
        assert_eq!(
            Request {
                mode: "permanent".into(),
                hours: None,
                minutes: None,
                seconds: None
            }
            .validate()
            .unwrap(),
            Policy::Permanent
        );
    }
    #[test]
    fn monotonic_start_deadline_freeze_restart_and_priority() {
        let mut c = Clock::new(Policy::Timed(1));
        c.refresh(3000);
        assert!(!c.started);
        assert_eq!(c.elapsed_seconds, 0);
        c.begin(4000).unwrap();
        assert!(!c.expired(4999));
        assert!(c.expired(5000));
        c.finish(5000, "时长已到");
        c.refresh(999999);
        assert_eq!(c.elapsed_seconds, 1);
        assert_eq!(c.remaining_seconds, Some(0));
        let mut c = Clock::new(Policy::Permanent);
        c.begin(u64::MAX - 10).unwrap();
        assert!(!c.expired(u64::MAX));
        let mut c = Clock::new(Policy::Timed(359999));
        assert!(c.begin(u64::MAX - 10).is_err());
        for phase in [
            "failed",
            "partial",
            "timeout",
            "cleanup-failed",
            "target-invalid",
        ] {
            assert_eq!(interruption(phase, true, true), None);
        }
        for phase in ["connecting", "writing", "draining", "waiting"] {
            assert_eq!(interruption(phase, false, true), Some("expired"));
        }
    }
}
