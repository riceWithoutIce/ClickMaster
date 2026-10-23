//! Session-only channel selection. No persisted preference or automatic fallback.
use serde::Serialize;

pub const MESSAGE: &str = "message";
pub const IPC: &str = "taptaploot-ipc";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub selected: String,
    pub ipc_available: bool,
    pub reason: String,
    pub epoch: u64,
}
impl Default for View {
    fn default() -> Self {
        Self {
            selected: MESSAGE.into(),
            ipc_available: false,
            reason: "请绑定支持版本的 TapTapLoot 窗口。".into(),
            epoch: u64::MAX,
        }
    }
}
pub fn idle(busy: bool, stopping: bool) -> Result<(), String> {
    if stopping {
        Err("程序正在退出。".into())
    } else if busy {
        Err("单次任务进行中，请先停止或等待结束。".into())
    } else {
        Ok(())
    }
}
pub fn select(view: &mut View, channel: &str) -> Result<(), String> {
    match channel {
        MESSAGE => {}
        IPC if view.ipc_available => {}
        IPC => return Err(view.reason.clone()),
        _ => return Err("未知执行通道。".into()),
    }
    view.selected = channel.into();
    Ok(())
}
/// Cached compatibility is presentation only. Explicit select/start always forces a check.
pub fn refresh(
    view: &mut View,
    epoch: u64,
    force: bool,
    check: impl FnOnce() -> Result<(), String>,
) {
    if force || view.epoch != epoch {
        let result = check();
        view.ipc_available = result.is_ok();
        view.reason = result
            .err()
            .unwrap_or_else(|| "兼容核验通过；开始执行时仍重新核验。".into());
        view.epoch = epoch;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_is_message_and_incompatible_ipc_never_falls_back() {
        let mut v = View::default();
        assert_eq!(v.selected, MESSAGE);
        assert!(select(&mut v, IPC).is_err());
        v.ipc_available = true;
        select(&mut v, IPC).unwrap();
        v.ipc_available = false;
        assert_eq!(v.selected, IPC);
        assert!(select(&mut v, "unknown").is_err());
    }
    #[test]
    fn mutations_require_idle_and_not_exiting() {
        assert!(idle(false, false).is_ok());
        assert!(idle(true, false).is_err());
        assert!(idle(false, true).is_err());
    }
    #[test]
    fn poll_reuses_binding_epoch_but_select_start_and_rebinding_revalidate() {
        let mut view = View::default();
        refresh(&mut view, 1, false, || Ok(()));
        refresh(&mut view, 1, false, || {
            panic!("poll must reuse compatibility")
        });
        refresh(&mut view, 1, true, || Err("forced start denied".into()));
        assert!(!view.ipc_available);
        assert_eq!(view.reason, "forced start denied");
        refresh(&mut view, 2, false, || Ok(()));
        assert!(view.ipc_available);
    }
}
