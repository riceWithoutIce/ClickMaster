//! Recoverable minimize policy, independent of target-window queries.
pub trait OwnWindow {
    fn taskbar(&self, skip: bool) -> Result<(), ()>;
    fn hide(&self) -> Result<(), ()>;
    fn show(&self) -> Result<(), ()>;
    fn minimize(&self) -> Result<(), ()>;
    fn unminimize(&self) -> Result<(), ()>;
}
pub fn tuck(window: &impl OwnWindow, keep: bool, tray: bool) -> Result<(), String> {
    if keep {
        window
            .taskbar(false)
            .map_err(|_| "无法保留任务栏入口，窗口保持打开。")?;
        return window
            .minimize()
            .map_err(|_| "无法最小化窗口，请重试。".into());
    }
    if !tray {
        return Err("托盘不可用，窗口保持打开；可取消或退出程序。".into());
    }
    if window.taskbar(true).is_err() || window.hide().is_err() {
        let _ = window.taskbar(false);
        let _ = window.show();
        let _ = window.unminimize();
        return Err("收起失败，已尝试恢复窗口和任务栏入口；仍可从托盘打开。".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct Fake {
        calls: RefCell<Vec<String>>,
        fail_hide: bool,
    }
    impl OwnWindow for Fake {
        fn taskbar(&self, skip: bool) -> Result<(), ()> {
            self.calls.borrow_mut().push(format!("taskbar:{skip}"));
            Ok(())
        }
        fn hide(&self) -> Result<(), ()> {
            self.calls.borrow_mut().push("hide".into());
            if self.fail_hide {
                Err(())
            } else {
                Ok(())
            }
        }
        fn show(&self) -> Result<(), ()> {
            self.calls.borrow_mut().push("show".into());
            Ok(())
        }
        fn minimize(&self) -> Result<(), ()> {
            self.calls.borrow_mut().push("minimize".into());
            Ok(())
        }
        fn unminimize(&self) -> Result<(), ()> {
            self.calls.borrow_mut().push("restore".into());
            Ok(())
        }
    }
    #[test]
    fn no_tray_never_hides_window() {
        let window = Fake {
            calls: RefCell::new(vec![]),
            fail_hide: false,
        };
        assert!(tuck(&window, false, false).is_err());
        assert!(window.calls.borrow().is_empty());
    }
    #[test]
    fn keep_taskbar_uses_normal_minimize() {
        let window = Fake {
            calls: RefCell::new(vec![]),
            fail_hide: false,
        };
        tuck(&window, true, true).unwrap();
        assert_eq!(*window.calls.borrow(), vec!["taskbar:false", "minimize"]);
    }
    #[test]
    fn hide_failure_restores_recovery_paths() {
        let window = Fake {
            calls: RefCell::new(vec![]),
            fail_hide: true,
        };
        assert!(tuck(&window, false, true).is_err());
        assert_eq!(
            *window.calls.borrow(),
            vec!["taskbar:true", "hide", "taskbar:false", "show", "restore"]
        );
    }
}
