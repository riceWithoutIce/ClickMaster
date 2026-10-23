//! Validated session range and unbiased inclusive integer sampling.
use serde::Deserialize;
pub const MAX_COMPLETED: u64 = 9_007_199_254_740_991;
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub min: Option<f64>,
    pub max: Option<f64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub min: u64,
    pub max: u64,
}
impl Request {
    pub fn validate(self, _ipc: bool) -> Result<Range, String> {
        // 1ms is an input boundary for stress testing. Windows scheduling and
        // transport latency still determine the achievable cadence.
        let lower = 1.0;
        let (min, max) = self.min.zip(self.max).ok_or("请填写最小和最大间隔。")?;
        if ![min, max]
            .iter()
            .all(|v| v.is_finite() && v.fract() == 0.0 && *v >= lower && *v <= 60000.0)
            || min > max
        {
            return Err(format!(
                "间隔必须为 {}～60000 毫秒整数，且最小不大于最大；不会自动修正。",
                lower as u32
            ));
        }
        Ok(Range {
            min: min as u64,
            max: max as u64,
        })
    }
}
impl Range {
    pub fn sample(self, mut random: impl FnMut() -> Result<u32, String>) -> Result<u64, String> {
        if self.min == self.max {
            return Ok(self.min);
        }
        let width = (self.max - self.min + 1) as u32;
        let threshold = width.wrapping_neg() % width;
        // Bounded rejection: a broken entropy source stops the task, never biases it.
        for _ in 0..128 {
            let x = random()?;
            if x >= threshold {
                return Ok(self.min + (x % width) as u64);
            }
        }
        Err("随机源持续返回拒绝值，执行已停止。".into())
    }
}
pub fn next_due(now: u64, delay: u64) -> Result<u64, String> {
    now.checked_add(delay)
        .ok_or("单调时钟超过安全范围，任务停止。".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges_reject_invalid_without_clamping() {
        assert_eq!(
            Request {
                min: Some(2000.0),
                max: Some(2000.0)
            }
            .validate(true)
            .unwrap(),
            Range {
                min: 2000,
                max: 2000
            }
        );
        for ipc in [false, true] {
            for bound in [1, 60000] {
                assert_eq!(
                    Request {
                        min: Some(bound as f64),
                        max: Some(bound as f64)
                    }
                    .validate(ipc)
                    .unwrap(),
                    Range {
                        min: bound,
                        max: bound
                    }
                );
            }
            for (min, max) in [
                (None, Some(2000.0)),
                (Some(0.5), Some(2000.0)),
                (Some(0.0), Some(100.0)),
                (Some(-1.0), Some(100.0)),
                (Some(100.5), Some(4000.0)),
                (Some(3000.0), Some(2000.0)),
                (Some(1.0), Some(60001.0)),
                (Some(f64::NAN), Some(60000.0)),
                (Some(f64::INFINITY), Some(60000.0)),
            ] {
                assert!(Request { min, max }.validate(ipc).is_err());
            }
        }
        assert!(Request {
            min: Some(100.0),
            max: Some(2000.0)
        }
        .validate(true)
        .is_ok());
        assert!(next_due(u64::MAX, 1).is_err());
    }
    #[test]
    fn fixed_does_not_use_entropy_and_rejection_is_unbiased() {
        assert_eq!(
            Range {
                min: 2000,
                max: 2000
            }
            .sample(|| panic!("fixed"))
            .unwrap(),
            2000
        );
        let range = Range { min: 100, max: 102 };
        let mut draws = vec![0, 1].into_iter();
        assert_eq!(range.sample(|| Ok(draws.next().unwrap())).unwrap(), 101); // 0 rejected for width3.
        let mut counts = [0; 3];
        for x in 1..=300 {
            counts[(range.sample(|| Ok(x)).unwrap() - 100) as usize] += 1;
        }
        assert_eq!(counts, [100, 100, 100]);
        assert!(range.sample(|| Ok(0)).is_err());
    }
    #[test]
    fn sustained_schedule_has_no_session_sixty_second_limit_or_catchup() {
        let mut now = 3000;
        for _ in 0..100 {
            now += 700; // Completed transfer, including scheduling/processing delay.
            let due = next_due(now, 2000).unwrap();
            assert_eq!(due - now, 2000);
            assert!(!crate::ipc::expired(700, 100, 0, "draining"));
            now = due + 500; // Late scheduler begins just one new operation.
        }
        assert!(now > crate::ipc::TOTAL_MS);
    }
}
