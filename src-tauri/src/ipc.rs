//! Fixed single-frame protocol and monotonic supervisor policy.
use serde::{Deserialize, Serialize};

fn is_zero(value: &u64) -> bool {
    *value == 0
}
pub const FRAME: [u8; 4] = [0, 2, 0x31, 0];
pub const PIPE: &str = r"\\.\pipe\TapTapLootxTheFarmerWasReplaced";
pub const ASSEMBLY_SHA256: &str =
    "BC5318AC557E4039ADCCAE8D79EAF66F6C0200D18FC043FD2AA632F6513C35B8";
pub const CONNECT_MS: u64 = 20_000;
pub const WRITE_MS: u64 = 2_000;
/// Best-effort read observation only; this is never a delivery success gate.
pub const READ_OBSERVE_MS: u64 = 200;
pub const TOTAL_MS: u64 = 60_000;

/// Closed vocabularies keep process identity, paths and OS error text out of reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureStage {
    Target,
    Compatibility,
    PipeCreate,
    Connect,
    Client,
    Write,
    Authorization,
    Worker,
    Status,
    Supervision,
    Cleanup,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureReason {
    ProcessUnavailable,
    ProcessExited,
    InstanceChanged,
    VersionUnreadable,
    UnsupportedVersion,
    ProcessEnumeration,
    MultipleInstances,
    WindowEnumeration,
    WindowAmbiguous,
    WindowInvalid,
    WindowIdentity,
    WindowHidden,
    WindowMinimized,
    EmptyClient,
    SystemCall,
    Timeout,
    ClientMismatch,
    WriteIncomplete,
    SetupFailed,
    AuthorizationFailed,
    WorkerExited,
    ReportInvalid,
    CleanupUnconfirmed,
    RandomUnavailable,
    CounterLimit,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Failure {
    pub stage: FailureStage,
    pub reason: FailureReason,
    pub error_code: Option<u32>,
}
impl Failure {
    pub fn new(stage: FailureStage, reason: FailureReason) -> Self {
        Self {
            stage,
            reason,
            error_code: None,
        }
    }
    pub fn code(mut self, code: u32) -> Self {
        self.error_code = Some(code);
        self
    }
    pub fn message(&self) -> &'static str {
        use FailureReason::*;
        match self.reason {
            ProcessUnavailable => "无法核验原目标进程。",
            ProcessExited => "原目标进程已退出，请重新绑定。",
            InstanceChanged => "原绑定进程实例已变化，请重新绑定。",
            VersionUnreadable => "无法读取支持版本文件。",
            UnsupportedVersion => "此游戏版本尚未支持专用 IPC。",
            ProcessEnumeration => "无法枚举目标实例。",
            MultipleInstances => "需要唯一的 TapTapLoot 进程实例。",
            WindowEnumeration => "无法枚举目标窗口。",
            WindowAmbiguous => "需要原实例唯一的 Unity 游戏窗口。",
            WindowInvalid => "原绑定窗口已失效。",
            WindowIdentity => "原窗口所属进程或线程已变化。",
            WindowHidden => "原游戏窗口不可见。",
            WindowMinimized => "原游戏窗口已最小化。",
            EmptyClient => "原游戏窗口客户区不可用。",
            SystemCall => "系统操作失败，请查看阶段与错误码。",
            Timeout => "操作超时，未自动重试。",
            ClientMismatch => "管道客户端不是原绑定游戏实例。",
            WriteIncomplete => "单帧未完整写入，未重试。",
            SetupFailed => "无法准备受控 IPC worker。",
            AuthorizationFailed => "worker 授权未完成。",
            WorkerExited => "worker 在完整写入前退出。",
            ReportInvalid => "worker 状态记录无效。",
            CleanupUnconfirmed => "自有资源清理未确认，执行已锁定。",
            RandomUnavailable => "随机源不可用。",
            CounterLimit => "发送计数已达到上限。",
        }
    }
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}
impl From<Failure> for String {
    fn from(value: Failure) -> Self {
        value.to_string()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReadStatus {
    #[default]
    Pending,
    Confirmed,
    Unconfirmed,
    Failed,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Receipt {
    pub connected: bool,
    pub written: bool,
    pub read_status: ReadStatus,
    pub read_error_code: Option<u32>,
    #[serde(default)]
    pub failure: Option<Failure>,
}
impl Receipt {
    pub fn fail(&mut self, failure: Failure) {
        if self.failure.is_none() {
            self.failure = Some(failure);
        }
    }
    pub fn finish_observation(&mut self) {
        if self.written && self.read_status == ReadStatus::Pending {
            self.read_status = ReadStatus::Unconfirmed;
        }
    }
}
/// Count the sole complete write, including when stop/deadline happens during cleanup.
pub fn record_sent(count: &mut u64, receipt: &Receipt) -> bool {
    if !receipt.written {
        return true;
    }
    if let Some(next) = count
        .checked_add(1)
        .filter(|n| *n <= crate::repeat::MAX_COMPLETED)
    {
        *count = next;
        true
    } else {
        false
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub phase: String,
    pub elapsed_ms: u64,
    pub stage_ms: u64,
    pub receipt: Receipt,
    /// Monotonic frame number within a persistent worker. Legacy single-frame
    /// workers leave this at zero.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance: Option<crate::performance::Metrics>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub performance_epoch: u64,
}
impl Report {
    pub fn new(phase: &str) -> Self {
        Self {
            phase: phase.into(),
            elapsed_ms: 0,
            stage_ms: 0,
            receipt: Receipt::default(),
            sequence: 0,
            performance: None,
            performance_epoch: 0,
        }
    }
}
pub fn phase_limit(phase: &str) -> Option<u64> {
    match phase {
        "connecting" => Some(CONNECT_MS),
        "writing" => Some(WRITE_MS),
        "checking" => Some(WRITE_MS),
        _ => None,
    }
}
pub fn expired(total: u64, observed_stage_age: u64, worker_stage_age: u64, phase: &str) -> bool {
    total >= TOTAL_MS
        || phase_limit(phase).is_some_and(|limit| observed_stage_age.max(worker_stage_age) >= limit)
}
pub fn phase_message(phase: &str) -> &'static str {
    match phase {
        "countdown" => "3秒准备；游戏可在前台或后台，请保持非最小化。",
        "waiting" => "单帧已发送；等待下一次，读取诊断不阻断执行。",
        "stopping" => "正在停止并清理自有传输资源；已发帧无法撤回。",
        "checking" => "正在核验原绑定实例与支持版本。",
        "connecting" => "等待原游戏实例连接专用 IPC。",
        "writing" => "客户端身份已核验，正在写入单帧。",
        "draining" => "单帧已发送，正在短时收尾；读取未确认不影响后续发送。",
        "completed" => "单帧已发送；读取结果仅供诊断，不代表游戏玩法响应。",
        "cancelled" => "任务已停止；已发帧无法撤回，不代表游戏未响应。",
        "timeout" => "任务超时，已清理自有 worker；不自动重试。",
        "cleanup-failed" => {
            "自有 worker 或本轮临时资源清理未确认，执行保持锁定；请退出程序后检查。"
        }
        _ => "IPC 未完成，已停止；请检查目标状态，不自动重试。",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_is_one_utf16_digit_with_big_endian_length() {
        assert_eq!(u16::from_be_bytes([FRAME[0], FRAME[1]]), 2);
        assert_eq!(u16::from_le_bytes([FRAME[2], FRAME[3]]), '1' as u16);
        assert_eq!(FRAME.len(), 4);
    }
    #[test]
    fn boundaries_use_monotonic_stage_age_and_total_budget() {
        for (phase, limit) in [("connecting", CONNECT_MS), ("writing", WRITE_MS)] {
            assert!(!expired(100, limit - 1, 0, phase));
            assert!(expired(100, limit, 0, phase));
            assert!(expired(100, 0, limit, phase));
        }
        assert!(expired(TOTAL_MS, 0, 0, "checking"));
        assert!(!expired(500, 500, 500, "draining"));
    }
}
