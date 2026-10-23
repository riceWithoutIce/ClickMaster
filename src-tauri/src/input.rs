//! Single-shot message construction and scheduler; all side effects are injected.
use crate::routing::{Snapshot, View};
use serde::{Deserialize, Serialize};

pub type Binding = Snapshot;
pub const COUNTDOWN_MS: u64 = 3000;
pub const RELEASE_MS: u64 = 50;

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub mode: String,
    pub key: String,
    pub center: bool,
    pub x: Option<f64>,
    pub y: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Left,
    Right,
    Key(u32),
}
pub fn action(request: &Request) -> Result<Action, String> {
    match request.mode.as_str() {
        "left" => Ok(Action::Left),
        "right" => Ok(Action::Right),
        "keyboard" => key(&request.key)
            .map(Action::Key)
            .ok_or("按键不在允许的单键列表中。".into()),
        _ => Err("未知输入方式。".into()),
    }
}
pub fn key(value: &str) -> Option<u32> {
    match value {
        "Space" => Some(0x20),
        "Enter" => Some(0x0d),
        "ArrowLeft" => Some(0x25),
        "ArrowUp" => Some(0x26),
        "ArrowRight" => Some(0x27),
        "ArrowDown" => Some(0x28),
        _ if value.len() == 1
            && value
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) =>
        {
            Some(value.as_bytes()[0] as u32)
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    pub id: u32,
    pub wparam: usize,
    pub lparam: isize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair {
    pub down: Message,
    pub up: Message,
}
#[cfg(test)]
pub fn mouse_pair(
    right: bool,
    width: i32,
    height: i32,
    request: &Request,
) -> Result<Pair, Failure> {
    let (x, y) = mouse_point(width, height, request)?;
    mouse_pair_at(right, width, height, (x, y))
}
pub fn mouse_point(width: i32, height: i32, request: &Request) -> Result<(i32, i32), Failure> {
    if width <= 0 || height <= 0 {
        return Err(Failure::reason("客户区不可用，未发送。"));
    }
    let point = if request.center {
        Some((width as f64 / 2.0).floor()).zip(Some((height as f64 / 2.0).floor()))
    } else {
        request.x.zip(request.y)
    };
    let (x, y) = point
        .filter(|(x, y)| {
            x.is_finite()
                && y.is_finite()
                && x.fract() == 0.0
                && y.fract() == 0.0
                && *x >= 0.0
                && *y >= 0.0
                && *x < width as f64
                && *y < height as f64
                && *x <= i32::MAX as f64
                && *y <= i32::MAX as f64
        })
        .ok_or_else(|| Failure::reason("坐标需为绑定客户区内的非负整数，未发送。"))?;
    Ok((x as i32, y as i32))
}
pub fn mouse_pair_at(
    right: bool,
    width: i32,
    height: i32,
    point: (i32, i32),
) -> Result<Pair, Failure> {
    let (x, y) = point;
    if width <= 0
        || height <= 0
        || x < 0
        || y < 0
        || x >= width
        || y >= height
        || x > i16::MAX as i32
        || y > i16::MAX as i32
    {
        return Err(Failure::reason(
            "映射后坐标不在接收客户区内或超过32767，未发送。",
        ));
    }
    let packed = ((x as u32) | ((y as u32) << 16)) as i32 as isize;
    Ok(Pair {
        down: Message {
            id: if right { 0x204 } else { 0x201 },
            wparam: if right { 2 } else { 1 },
            lparam: packed,
        },
        up: Message {
            id: if right { 0x205 } else { 0x202 },
            wparam: 0,
            lparam: packed,
        },
    })
}
pub fn routed_mouse_pair(
    right: bool,
    root_size: (i32, i32),
    receiver_size: (i32, i32),
    request: &Request,
    map: impl FnOnce((i32, i32)) -> Result<(i32, i32), Failure>,
) -> Result<Pair, Failure> {
    let original = mouse_point(root_size.0, root_size.1, request)?;
    let mapped = map(original)?;
    mouse_pair_at(right, receiver_size.0, receiver_size.1, mapped)
}
pub fn keyboard_pair(vk: u32, scan: u32) -> Result<Pair, Failure> {
    let prefix = (scan >> 8) & 0xff;
    if scan & 0xff == 0 || scan > 0xffff || (prefix != 0 && prefix != 0xe0) {
        return Err(Failure::reason("无法确定该单键的扫描码，未发送。"));
    }
    let bits = 1u32 | ((scan & 0xff) << 16) | if prefix == 0xe0 { 1 << 24 } else { 0 };
    Ok(Pair {
        down: Message {
            id: 0x100,
            wparam: vk as usize,
            lparam: bits as isize,
        },
        up: Message {
            id: 0x101,
            wparam: vk as usize,
            lparam: (bits | 0xc0000000) as i32 as isize,
        },
    })
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    pub reason: String,
    pub error_code: Option<u32>,
}
impl Failure {
    pub fn reason(reason: &str) -> Self {
        Self {
            reason: reason.into(),
            error_code: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub posted: bool,
    pub error: Option<Failure>,
}
impl Receipt {
    fn from(result: Result<(), Failure>) -> Self {
        match result {
            Ok(()) => Self {
                posted: true,
                error: None,
            },
            Err(error) => Self {
                posted: false,
                error: Some(error),
            },
        }
    }
}

pub trait Adapter {
    fn valid(&mut self, binding: &Binding) -> bool;
    fn prepare(
        &mut self,
        binding: &Binding,
        action: &Action,
        request: &Request,
    ) -> Result<Pair, Failure>;
    fn post(&mut self, binding: &Binding, message: Message) -> Result<(), Failure>;
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub repeating: bool,
    pub timing: crate::duration::Clock,
    pub completed: u64,
    pub channel: String,
    pub ipc: Option<crate::ipc::Receipt>,
    pub revision: u64,
    pub task_id: u64,
    pub phase: String,
    pub busy: bool,
    pub remaining_ms: u64,
    pub message: String,
    pub summary: String,
    pub receiver: Option<View>,
    pub down: Option<Receipt>,
    pub up: Option<Receipt>,
}
impl Default for Status {
    fn default() -> Self {
        Self {
            repeating: false,
            timing: crate::duration::Clock::default(),
            completed: 0,
            channel: "message".into(),
            ipc: None,
            revision: 0,
            task_id: 0,
            phase: "idle".into(),
            busy: false,
            remaining_ms: 0,
            message: "待命；开始前将注册全局停止 F8。".into(),
            summary: String::new(),
            receiver: None,
            down: None,
            up: None,
        }
    }
}
#[derive(Clone)]
struct Job {
    range: Option<crate::repeat::Range>,
    binding: Binding,
    action: Action,
    request: Request,
    due: u64,
    release: Option<Message>,
}
#[derive(Default)]
pub struct Engine {
    pub status: Status,
    job: Option<Job>,
    last_now: u64,
}
impl Engine {
    fn phase(&mut self, phase: &str, message: &str, busy: bool) {
        self.status.revision += 1;
        self.status.phase = phase.into();
        self.status.message = message.into();
        self.status.busy = busy;
        if !busy {
            self.status.remaining_ms = 0;
        }
    }
    #[cfg(test)]
    pub fn start(
        &mut self,
        now: u64,
        binding: Binding,
        request: Request,
    ) -> Result<Status, String> {
        self.start_task(now, binding, request, None)
    }
    pub fn start_task(
        &mut self,
        now: u64,
        binding: Binding,
        request: Request,
        range: Option<crate::repeat::Range>,
    ) -> Result<Status, String> {
        if self.job.is_some() || self.status.busy {
            return Err("已有单次任务；不会并发或排队，请等待或取消。".into());
        }
        let action = action(&request)?;
        let due = crate::repeat::next_due(now, COUNTDOWN_MS)?;
        self.status.channel = "message".into();
        self.status.ipc = None;
        self.status.receiver = Some(binding.view.clone());
        self.status.summary = match &action {
            Action::Key(_) => format!("键盘 {}", request.key),
            mouse => format!(
                "{} · {}",
                if *mouse == Action::Right {
                    "鼠标右键"
                } else {
                    "鼠标左键"
                },
                if request.center {
                    "当前客户区中心"
                } else {
                    "指定客户区坐标"
                }
            ),
        };
        self.status.task_id = self
            .status
            .task_id
            .checked_add(1)
            .ok_or("任务计数达到安全上限。")?;
        self.status.repeating = range.is_some();
        self.status.timing = crate::duration::Clock::default();
        self.last_now = now;
        self.status.completed = 0;
        self.status.down = None;
        self.status.up = None;
        self.status.remaining_ms = COUNTDOWN_MS;
        self.job = Some(Job {
            range,
            binding,
            action,
            request,
            due,
            release: None,
        });
        self.phase(
            "countdown",
            "3秒倒计时，切到其他工作窗口后等待投递；可取消。",
            true,
        );
        Ok(self.status.clone())
    }
    #[cfg(test)]
    pub fn tick(&mut self, now: u64, adapter: &mut impl Adapter) {
        self.tick_with_random(now, adapter, crate::ipc_native::random_u32);
    }
    #[cfg(test)]
    pub fn tick_with_random(
        &mut self,
        now: u64,
        adapter: &mut impl Adapter,
        random: impl FnMut() -> Result<u32, String>,
    ) {
        self.tick_with_clock(now, adapter, random, || now);
    }
    pub fn tick_with_clock(
        &mut self,
        now: u64,
        adapter: &mut impl Adapter,
        mut random: impl FnMut() -> Result<u32, String>,
        mut finished_at: impl FnMut() -> u64,
    ) {
        self.last_now = now;
        let previous_timing = self.status.timing.clone();
        self.tick_inner(now, adapter, &mut random, &mut finished_at);
        let now = finished_at().max(now);
        self.last_now = now;
        self.status.timing.refresh(now);
        if !self.status.busy && self.status.task_id > 0 {
            let reason = self
                .status
                .timing
                .end_reason
                .clone()
                .unwrap_or_else(|| crate::duration::reason(&self.status.phase).into());
            self.status.timing.finish(now, &reason);
        }
        if self.status.timing != previous_timing {
            self.status.revision += 1;
        }
    }
    fn tick_inner(
        &mut self,
        now: u64,
        adapter: &mut impl Adapter,
        mut random: impl FnMut() -> Result<u32, String>,
        mut finished_at: impl FnMut() -> u64,
    ) {
        let Some(job) = &self.job else {
            return;
        };
        if !adapter.valid(&job.binding) {
            self.cancel(adapter);
            if self.status.phase == "cancelled" {
                self.phase(
                    "cancelled",
                    "原绑定已变化或无法验证，任务取消，未发送。",
                    false,
                );
            }
            if self.status.phase == "cancelled" {
                self.status.timing.end_reason = Some("目标失效".into());
                self.status.timing.trigger_reason = None;
            }
            return;
        }
        if self.status.phase == "countdown" && now >= job.due {
            if let Err(error) = self.status.timing.begin(now) {
                self.job = None;
                self.phase("failed", &error, false);
                return;
            }
        }
        if self.status.timing.expired(now) {
            self.expire(now, adapter);
            return;
        }
        let job = self.job.as_ref().unwrap();
        if now < job.due {
            let remaining = job.due - now;
            if matches!(self.status.phase.as_str(), "countdown" | "waiting")
                && self.status.remaining_ms.div_ceil(100) != remaining.div_ceil(100)
            {
                self.status.remaining_ms = remaining;
                self.status.revision += 1;
            }
            return;
        }
        if job.release.is_some() {
            let job = job.clone();
            self.release(adapter, false);
            if self.status.phase == "delivered" {
                if self.status.timing.expired(finished_at()) {
                    self.phase(
                        "expired",
                        "时长已到，配对释放已完成；不再开始新操作。",
                        false,
                    );
                    return;
                }
                if let Some(range) = job.range {
                    let next = range.sample(&mut random).and_then(|delay| {
                        crate::repeat::next_due(finished_at(), delay).map(|due| (due, delay))
                    });
                    match next {
                        Ok((due, delay)) => {
                            let mut next_job = job.clone();
                            next_job.release = None;
                            next_job.due = due;
                            self.job = Some(next_job);
                            self.status.remaining_ms = delay;
                            self.phase(
                                "waiting",
                                "本次按下及释放已投递；正在等待下一次，玩法响应待人工确认。",
                                true,
                            );
                        }
                        Err(error) => self.phase("failed", &error, false),
                    }
                }
            }
            return;
        }
        if self.status.completed >= crate::repeat::MAX_COMPLETED
            || crate::repeat::next_due(now, RELEASE_MS).is_err()
        {
            self.job = None;
            self.phase("failed", "计数或时钟达到安全上限，未开始新操作。", false);
            return;
        }
        let pair = adapter.prepare(&job.binding, &job.action, &job.request);
        let pair = match pair {
            Ok(pair) => pair,
            Err(error) => {
                self.status.down = Some(Receipt::from(Err(error)));
                self.job = None;
                self.phase("rejected", "参数或目标状态不可用，未发送。", false);
                return;
            }
        };
        let job = self.job.as_ref().unwrap();
        if !adapter.valid(&job.binding) {
            self.cancel(adapter);
            self.phase("cancelled", "按下前原绑定已变化或无法验证，未发送。", false);
            self.status.timing.end_reason = Some("目标失效".into());
            self.status.timing.trigger_reason = None;
            return;
        }
        let before_send = finished_at();
        if self.status.timing.expired(before_send) {
            self.last_now = before_send;
            self.expire(before_send, adapter);
            return;
        }
        let job = self.job.as_ref().unwrap();
        let posted = adapter.post(&job.binding, pair.down);
        self.status.up = None;
        let succeeded = posted.is_ok();
        self.status.down = Some(Receipt::from(posted));
        if !succeeded {
            self.job = None;
            self.phase("failed", "按下投递失败；未投递释放，不自动重试。", false);
            return;
        }
        let job = self.job.as_mut().unwrap();
        job.release = Some(pair.up);
        job.due = match crate::repeat::next_due(now, RELEASE_MS) {
            Ok(due) => due,
            Err(_) => {
                self.release(adapter, true);
                return;
            }
        };
        self.status.remaining_ms = 0;
        self.phase(
            "sending",
            "按下已投递，正在完成配对释放；消息无法撤回。",
            true,
        );
    }
    fn release(&mut self, adapter: &mut impl Adapter, cancelled: bool) {
        let Some(job) = self.job.take() else {
            return;
        };
        let Some(up) = job.release else {
            return;
        };
        let result = if adapter.valid(&job.binding) {
            adapter.post(&job.binding, up)
        } else {
            Err(Failure::reason("原绑定身份失效，禁止向替代窗口投递释放。"))
        };
        let succeeded = result.is_ok();
        self.status.up = Some(Receipt::from(result));
        if succeeded {
            let Some(count) = self.status.completed.checked_add(1) else {
                self.phase("failed", "完成计数达到安全上限，任务停止。", false);
                return;
            };
            self.status.completed = count;
        }
        if !succeeded {
            self.phase(
                "partial",
                "部分投递：按下已投递，释放失败或原身份失效。请人工检查目标状态。",
                false,
            );
        } else if cancelled {
            self.phase(
                "cancelled",
                "已取消后续等待；原按下已完成配对释放，已入队消息无法撤回。",
                false,
            );
        } else {
            self.phase(
                "delivered",
                "按下及释放已投递，目标是否响应请人工确认。",
                false,
            );
        }
    }
    pub fn expire(&mut self, now: u64, adapter: &mut impl Adapter) {
        self.last_now = now;
        self.cancel(adapter);
        self.status.timing.trigger_reason = Some("时长已到".into());
        if self.status.phase == "cancelled" {
            self.phase(
                "expired",
                "时长已到，已停止后续操作；已投递消息无法撤回。",
                false,
            );
            self.status.timing.end_reason = Some("时长已到".into());
        }
        self.status
            .timing
            .finish(now, crate::duration::reason(&self.status.phase));
    }
    pub fn cancel_at(&mut self, now: u64, adapter: &mut impl Adapter) -> Status {
        self.last_now = now;
        self.cancel(adapter)
    }
    pub fn cancel(&mut self, adapter: &mut impl Adapter) -> Status {
        if self.job.as_ref().is_some_and(|job| job.release.is_some()) {
            self.release(adapter, true);
        } else if self.job.take().is_some() {
            self.phase("cancelled", "已停止准备或下次等待；不会开始新操作。", false);
        }
        if !self.status.busy {
            self.status
                .timing
                .finish(self.last_now, crate::duration::reason(&self.status.phase));
        }
        self.status.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::targets::Identity;
    #[test]
    fn timed_wait_deadline_stops_before_long_interval_and_summary_freezes() {
        let mut e = Engine::default();
        let mut io = Fake::default();
        e.start_task(
            0,
            binding(),
            request("left"),
            Some(crate::repeat::Range {
                min: 60000,
                max: 60000,
            }),
        )
        .unwrap();
        e.status.timing = crate::duration::Clock::new(crate::duration::Policy::Timed(1));
        e.tick(2999, &mut io);
        assert!(!e.status.timing.started);
        assert!(io.sent.is_empty());
        e.tick(3000, &mut io);
        e.tick(3050, &mut io);
        assert_eq!(e.status.completed, 1);
        e.tick(3999, &mut io);
        assert!(e.status.busy);
        e.tick(4000, &mut io);
        assert_eq!(e.status.phase, "expired");
        assert_eq!(e.status.timing.end_reason.as_deref(), Some("时长已到"));
        assert_eq!(io.sent.len(), 2);
        let summary = e.status.timing.clone();
        e.tick(999999, &mut io);
        assert_eq!(e.status.timing, summary);
        e.start(1000000, binding(), request("left")).unwrap();
        assert_eq!(e.status.completed, 0);
        assert!(!e.status.timing.started);
        assert!(e.status.timing.end_reason.is_none());
    }
    #[test]
    fn expiry_releases_only_original_and_partial_failure_wins() {
        for fail in [false, true] {
            let mut e = Engine::default();
            let mut io = Fake::default();
            e.start_task(
                0,
                binding(),
                request("left"),
                Some(crate::repeat::Range { min: 100, max: 100 }),
            )
            .unwrap();
            e.status.timing = crate::duration::Clock::new(crate::duration::Policy::Timed(1));
            e.tick(3000, &mut io);
            if fail {
                io.binding = None;
            }
            e.tick(4000, &mut io);
            assert!(!e.status.busy);
            assert_eq!(e.status.phase, if fail { "partial" } else { "expired" });
            assert_eq!(io.sent.len(), if fail { 1 } else { 2 });
            assert_eq!(
                e.status.timing.end_reason.as_deref(),
                Some(if fail { "部分投递" } else { "时长已到" })
            );
        }
    }
    #[test]
    fn deadline_rechecked_after_prepare_and_operation_completion() {
        let mut e = Engine::default();
        let mut io = Fake::default();
        e.start_task(
            0,
            binding(),
            request("left"),
            Some(crate::repeat::Range { min: 100, max: 100 }),
        )
        .unwrap();
        e.status.timing = crate::duration::Clock::new(crate::duration::Policy::Timed(1));
        e.tick_with_clock(3000, &mut io, || panic!("not needed"), || 4000);
        assert!(io.sent.is_empty());
        assert_eq!(e.status.phase, "expired");
        assert_eq!(e.status.timing.elapsed_seconds, 1);
        let mut e = Engine::default();
        e.start_task(
            0,
            binding(),
            request("left"),
            Some(crate::repeat::Range { min: 100, max: 100 }),
        )
        .unwrap();
        e.status.timing = crate::duration::Clock::new(crate::duration::Policy::Timed(1));
        e.tick(3000, &mut io);
        e.tick_with_clock(3050, &mut io, || panic!("expired before random"), || 4000);
        assert_eq!(io.sent.len(), 2);
        assert_eq!(e.status.phase, "expired");
    }
    #[test]
    fn manual_stop_freezes_elapsed_and_permanent_has_no_deadline() {
        let mut e = Engine::default();
        let mut io = Fake::default();
        e.start_task(
            0,
            binding(),
            request("left"),
            Some(crate::repeat::Range {
                min: 60000,
                max: 60000,
            }),
        )
        .unwrap();
        e.tick(3000, &mut io);
        e.tick(3050, &mut io);
        e.tick(63050, &mut io);
        e.tick(63100, &mut io);
        assert!(e.status.busy);
        assert_eq!(e.status.completed, 2);
        assert_eq!(e.status.timing.limit_seconds, None);
        e.cancel_at(64000, &mut io);
        assert_eq!(e.status.timing.elapsed_seconds, 61);
        let old = e.status.timing.clone();
        e.tick(999999, &mut io);
        assert_eq!(e.status.timing, old);
    }
    #[test]
    fn repeat_waits_after_release_without_catchup_and_stop_prevents_future_down() {
        let mut engine = Engine::default();
        let mut io = Fake::default();
        engine
            .start_task(
                0,
                binding(),
                request("left"),
                Some(crate::repeat::Range { min: 100, max: 100 }),
            )
            .unwrap();
        assert!(engine.start(0, binding(), request("right")).is_err());
        engine.tick_with_random(3000, &mut io, || panic!("fixed"));
        assert_eq!(io.sent.len(), 1);
        engine.tick_with_random(9000, &mut io, || panic!("fixed"));
        assert_eq!(io.sent.len(), 2);
        assert_eq!(engine.status.phase, "waiting");
        assert_eq!(engine.status.completed, 1);
        engine.tick_with_random(9099, &mut io, || panic!("fixed"));
        assert_eq!(io.sent.len(), 2);
        engine.tick_with_random(9100, &mut io, || panic!("fixed"));
        assert_eq!(io.sent.len(), 3);
        engine.cancel(&mut io);
        assert_eq!(io.sent.len(), 4);
        assert!(!engine.status.busy);
        assert_eq!(engine.status.completed, 2);
        engine.tick(999999, &mut io);
        assert_eq!(io.sent.len(), 4);
    }
    #[test]
    fn waiting_starts_at_actual_operation_completion_and_clock_limits_never_send() {
        let mut engine = Engine::default();
        let mut io = Fake::default();
        engine
            .start_task(
                0,
                binding(),
                request("left"),
                Some(crate::repeat::Range { min: 100, max: 100 }),
            )
            .unwrap();
        engine.tick(3000, &mut io);
        engine.tick_with_clock(3050, &mut io, || panic!("fixed"), || 4000);
        engine.tick(4099, &mut io);
        assert_eq!(io.sent.len(), 2);
        engine.tick(4100, &mut io);
        assert_eq!(io.sent.len(), 3);
        engine.cancel(&mut io);
        let mut engine = Engine::default();
        let mut io = Fake::default();
        assert!(engine.start(u64::MAX, binding(), request("left")).is_err());
        assert!(!engine.status.busy);
        engine.start(0, binding(), request("left")).unwrap();
        engine.status.completed = crate::repeat::MAX_COMPLETED;
        engine.tick(3000, &mut io);
        assert!(io.sent.is_empty());
        assert!(!engine.status.busy);
    }
    #[test]
    fn random_is_sampled_after_each_completion_and_waiting_identity_failure_stops() {
        let mut engine = Engine::default();
        let mut io = Fake::default();
        let mut draws = 0;
        engine
            .start_task(
                0,
                binding(),
                request("left"),
                Some(crate::repeat::Range { min: 100, max: 102 }),
            )
            .unwrap();
        engine.tick_with_random(3000, &mut io, || panic!("before completion"));
        engine.tick_with_random(3050, &mut io, || {
            draws += 1;
            Ok(2)
        });
        assert_eq!(draws, 1);
        assert_eq!(engine.status.remaining_ms, 102);
        io.binding = None;
        engine.tick(3100, &mut io);
        assert!(!engine.status.busy);
        assert_eq!(io.sent.len(), 2);
        let mut engine = Engine::default();
        let mut io = Fake::default();
        io.fail_at = Some(1);
        engine
            .start_task(
                0,
                binding(),
                request("left"),
                Some(crate::repeat::Range { min: 100, max: 200 }),
            )
            .unwrap();
        engine.tick(3000, &mut io);
        engine.tick(3050, &mut io);
        assert_eq!(engine.status.phase, "partial");
        assert!(!engine.status.busy);
        assert_eq!(engine.status.completed, 0);
    }
    #[test]
    fn routed_coordinates_validate_both_spaces_and_mapping_failure() {
        let req = request("left");
        let pair =
            routed_mouse_pair(false, (100, 80), (100, 60), &req, |(x, y)| Ok((x, y - 20))).unwrap();
        assert_eq!(pair.down.id, 0x201);
        assert_eq!(pair.down.lparam, 50 | (20 << 16));
        let mut custom = req.clone();
        custom.center = false;
        custom.x = Some(0.0);
        custom.y = Some(5.0);
        assert!(
            routed_mouse_pair(true, (100, 80), (100, 60), &custom, |(x, y)| Ok((
                x,
                y - 20
            )))
            .is_err()
        );
        assert!(routed_mouse_pair(true, (100, 80), (10, 10), &req, Ok).is_err());
        assert!(
            routed_mouse_pair(false, (100, 80), (100, 80), &req, |_| Err(Failure::reason(
                "map failed"
            )))
            .is_err()
        );
        custom.x = Some(100.0);
        assert!(routed_mouse_pair(false, (100, 80), (200, 200), &custom, Ok).is_err());
        custom.x = Some(32767.0);
        custom.y = Some(0.0);
        assert!(routed_mouse_pair(true, (40000, 100), (40000, 100), &custom, Ok).is_ok());
        custom.x = Some(32768.0);
        assert!(routed_mouse_pair(true, (40000, 100), (40000, 100), &custom, Ok).is_err());
    }
    fn binding() -> Binding {
        let identity = Identity {
            hwnd: 1,
            pid: 2,
            created: 3,
            thread: 4,
        };
        Snapshot {
            root: (identity.clone(), 5),
            receiver: identity,
            epoch: 5,
            view: View {
                token: "fixture".into(),
                exe: "Fixture.exe".into(),
                pid: 2,
                class: "FixtureWindow".into(),
                width: 1001,
                height: 501,
                is_self: true,
            },
        }
    }
    fn request(mode: &str) -> Request {
        Request {
            mode: mode.into(),
            key: "Space".into(),
            center: true,
            x: None,
            y: None,
        }
    }
    struct Fake {
        binding: Option<Binding>,
        sent: Vec<Message>,
        size: (i32, i32),
        fail_at: Option<usize>,
        minimized: bool,
        invalidate_on_prepare: bool,
    }
    impl Default for Fake {
        fn default() -> Self {
            Self {
                binding: Some(binding()),
                sent: vec![],
                size: (1001, 501),
                fail_at: None,
                minimized: false,
                invalidate_on_prepare: false,
            }
        }
    }
    impl Adapter for Fake {
        fn valid(&mut self, expected: &Binding) -> bool {
            self.binding.as_ref() == Some(expected)
        }
        fn prepare(
            &mut self,
            _: &Binding,
            action: &Action,
            request: &Request,
        ) -> Result<Pair, Failure> {
            if self.invalidate_on_prepare {
                self.binding.as_mut().unwrap().epoch += 1;
            }
            if self.minimized {
                return Err(Failure::reason("minimized"));
            }
            match action {
                Action::Key(vk) => keyboard_pair(*vk, 0x39),
                other => mouse_pair(*other == Action::Right, self.size.0, self.size.1, request),
            }
        }
        fn post(&mut self, _: &Binding, message: Message) -> Result<(), Failure> {
            let index = self.sent.len();
            self.sent.push(message);
            if self.fail_at == Some(index) {
                Err(Failure {
                    reason: "fake post failure".into(),
                    error_code: Some(5),
                })
            } else {
                Ok(())
            }
        }
    }
    #[test]
    fn mouse_messages_center_edges_and_signed_sixteen_bit_validation() {
        for right in [false, true] {
            let pair = mouse_pair(right, 1001, 501, &request("left")).unwrap();
            assert_eq!(pair.down.id, if right { 0x204 } else { 0x201 });
            assert_eq!(pair.up.id, pair.down.id + 1);
            assert_eq!(pair.down.wparam, if right { 2 } else { 1 });
            assert_eq!(pair.up.wparam, 0);
            assert_eq!(pair.down.lparam, 500 | (250 << 16));
            assert_eq!(pair.up.lparam, pair.down.lparam);
        }
        let mut r = request("left");
        r.center = false;
        for (x, y, valid) in [
            (0.0, 0.0, true),
            (32767.0, 32767.0, true),
            (32768.0, 0.0, false),
            (-1.0, 0.0, false),
            (1.5, 0.0, false),
            (f64::NAN, 0.0, false),
        ] {
            r.x = Some(x);
            r.y = Some(y);
            assert_eq!(mouse_pair(false, 40000, 40000, &r).is_ok(), valid);
        }
        r.x = Some(100.0);
        r.y = Some(0.0);
        assert!(mouse_pair(false, 100, 100, &r).is_err());
        r.x = None;
        assert!(mouse_pair(false, 100, 100, &r).is_err());
        assert!(mouse_pair(false, 0, 100, &request("left")).is_err());
        assert!(mouse_pair(false, 70000, 100, &request("left")).is_err());
    }
    #[test]
    fn keyboard_whitelist_scancode_extended_and_release_bits() {
        assert_eq!(key("A"), Some(65));
        assert_eq!(key("Z"), Some(90));
        assert_eq!(key("0"), Some(48));
        assert_eq!(key("9"), Some(57));
        for name in [
            "Space",
            "Enter",
            "ArrowLeft",
            "ArrowUp",
            "ArrowRight",
            "ArrowDown",
        ] {
            assert!(key(name).is_some());
        }
        for name in ["a", "F1", "Alt", "Ctrl+A", "65", "", "é"] {
            assert!(key(name).is_none());
        }
        for (vk, scan, extended) in [
            (0x20, 0x39, false),
            (0x0d, 0x1c, false),
            (0x25, 0xe04b, true),
            (0x26, 0xe048, true),
            (0x27, 0xe04d, true),
            (0x28, 0xe050, true),
            (65, 0x1e, false),
            (48, 0x0b, false),
        ] {
            let p = keyboard_pair(vk, scan).unwrap();
            assert_eq!((p.down.id, p.up.id), (0x100, 0x101));
            assert_eq!(p.down.wparam, vk as usize);
            assert_eq!(p.up.wparam, p.down.wparam);
            assert_eq!(p.down.lparam as u32 & 0xffff, 1);
            assert_eq!((p.down.lparam as u32 >> 16) & 0xff, scan & 0xff);
            assert_eq!((p.down.lparam as u32 >> 24) & 1, u32::from(extended));
            assert_eq!(p.down.lparam as u32 >> 30, 0);
            assert_eq!(p.up.lparam as u32, p.down.lparam as u32 | 0xc0000000);
        }
        assert!(keyboard_pair(65, 0).is_err());
        assert!(keyboard_pair(65, 0xe11d).is_err());
        assert!(action(&Request {
            key: "F1".into(),
            ..request("keyboard")
        })
        .is_err());
    }
    #[test]
    fn countdown_snapshot_duplicate_rejection_and_fifty_ms_release() {
        let mut engine = Engine::default();
        let mut io = Fake::default();
        engine.start(100, binding(), request("right")).unwrap();
        assert!(engine.start(200, binding(), request("left")).is_err());
        engine.tick(3099, &mut io);
        assert!(io.sent.is_empty());
        engine.tick(3100, &mut io);
        assert_eq!(io.sent[0].id, 0x204);
        assert!(engine.status.busy);
        io.size = (2, 2);
        engine.tick(3149, &mut io);
        assert_eq!(io.sent.len(), 1);
        engine.tick(3150, &mut io);
        assert_eq!(io.sent.len(), 2);
        assert_eq!(io.sent[1].lparam, io.sent[0].lparam);
        assert_eq!(engine.status.phase, "delivered");
        assert!(!engine.status.busy);
        engine.tick(9999, &mut io);
        assert_eq!(io.sent.len(), 2);
    }
    #[test]
    fn cancel_exit_rebind_and_identity_changes_never_follow_replacement() {
        for after_down in [false, true] {
            for changed in [false, true] {
                let mut e = Engine::default();
                let mut io = Fake::default();
                e.start(0, binding(), request("left")).unwrap();
                if after_down {
                    e.tick(3000, &mut io);
                }
                if changed {
                    io.binding.as_mut().unwrap().epoch += 1;
                }
                e.cancel(&mut io);
                assert_eq!(
                    io.sent.len(),
                    if after_down {
                        if changed {
                            1
                        } else {
                            2
                        }
                    } else {
                        0
                    }
                );
                assert_eq!(
                    e.status.phase,
                    if after_down && changed {
                        "partial"
                    } else {
                        "cancelled"
                    }
                );
                assert!(!e.status.busy);
            }
        }
        for field in 0..7 {
            let mut e = Engine::default();
            let mut io = Fake::default();
            e.start(0, binding(), request("left")).unwrap();
            match field {
                0 => io.binding.as_mut().unwrap().receiver.pid += 1,
                1 => io.binding.as_mut().unwrap().receiver.created += 1,
                2 => io.binding.as_mut().unwrap().receiver.thread += 1,
                3 => io.binding.as_mut().unwrap().epoch += 1,
                4 => io.binding.as_mut().unwrap().root.0.created += 1,
                5 => io.binding.as_mut().unwrap().root.1 += 1,
                _ => io.binding = None,
            };
            e.tick(3000, &mut io);
            assert!(io.sent.is_empty());
            assert_eq!(e.status.phase, "cancelled");
        }
    }
    #[test]
    fn minimized_rejected_down_failed_and_release_partial_are_distinct() {
        for fail in [None, Some(0), Some(1)] {
            let mut e = Engine::default();
            let mut io = Fake {
                fail_at: fail,
                minimized: fail.is_none(),
                ..Default::default()
            };
            e.start(0, binding(), request("left")).unwrap();
            e.tick(3000, &mut io);
            e.tick(3050, &mut io);
            assert_eq!(
                e.status.phase,
                match fail {
                    None => "rejected",
                    Some(0) => "failed",
                    _ => "partial",
                }
            );
            assert!(!e.status.busy);
            if fail.is_some() {
                let receipt = if fail == Some(0) {
                    e.status.down.as_ref().unwrap()
                } else {
                    e.status.up.as_ref().unwrap()
                };
                assert_eq!(receipt.error.as_ref().unwrap().error_code, Some(5));
            }
        }
    }

    #[test]
    fn keyboard_job_uses_snapshot_and_terminal_task_never_repeats() {
        let mut e = Engine::default();
        let mut io = Fake::default();
        let mut draft = request("keyboard");
        e.start(0, binding(), draft.clone()).unwrap();
        draft.key = "Enter".into();
        assert_eq!(e.status.summary, "键盘 Space");
        e.tick(3000, &mut io);
        e.tick(3050, &mut io);
        assert_eq!(io.sent.len(), 2);
        assert_eq!(io.sent[0].wparam, 0x20);
        assert_eq!(io.sent[0].id, 0x100);
        assert_eq!(io.sent[1].id, 0x101);
        e.cancel(&mut io);
        e.tick(9000, &mut io);
        assert_eq!(io.sent.len(), 2);
        assert_eq!(e.status.phase, "delivered");
        assert!(e.start(9000, binding(), draft).is_ok());
        assert_eq!(e.status.task_id, 2);
    }
    #[test]
    fn identity_rechecked_after_prepare_and_before_release() {
        let mut e = Engine::default();
        let mut io = Fake {
            invalidate_on_prepare: true,
            ..Default::default()
        };
        e.start(0, binding(), request("left")).unwrap();
        e.tick(3000, &mut io);
        assert!(io.sent.is_empty());
        assert_eq!(e.status.phase, "cancelled");
        let mut e = Engine::default();
        let mut io = Fake::default();
        e.start(0, binding(), request("left")).unwrap();
        e.tick(3000, &mut io);
        io.binding.as_mut().unwrap().receiver.created += 1;
        e.tick(3010, &mut io);
        assert_eq!(io.sent.len(), 1);
        assert_eq!(e.status.phase, "partial");
        assert!(e.status.down.unwrap().posted);
        assert!(!e.status.up.unwrap().posted);
    }
}
