//! Backend-owned candidate identities. Display text is never accepted as identity.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Identity {
    pub hwnd: usize,
    pub pid: u32,
    pub created: u64,
    pub thread: u32,
}

#[derive(Clone, Debug)]
pub struct ObservedWindow {
    pub identity: Identity,
    pub title: String,
    pub exe: String,
    pub status: WindowStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowStatus {
    Visible,
    Minimized,
    Hidden,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryError {
    Gone,
    Permission,
    Failed,
}

pub struct Enumeration {
    pub windows: Vec<ObservedWindow>,
    pub inaccessible: usize,
}

pub trait WindowQuery {
    fn enumerate(&self, include_hidden: bool) -> Result<Enumeration, QueryError>;
    fn inspect(&self, hwnd: usize) -> Result<ObservedWindow, QueryError>;
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetView {
    pub token: String,
    pub title: String,
    pub exe: String,
    pub pid: u32,
    pub status: WindowStatus,
}

#[derive(Clone)]
struct Candidate {
    view: TargetView,
    identity: Identity,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingStatus {
    pub revision: u64,
    pub epoch: u64,
    pub bound: Option<TargetView>,
    pub reason: Option<String>,
    pub routing: crate::routing::Status,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetList {
    pub windows: Vec<TargetView>,
    pub inaccessible: usize,
    pub binding: BindingStatus,
}

#[derive(Default)]
pub struct TargetState {
    pub routing: crate::routing::State,
    candidates: Vec<Candidate>,
    bound: Option<Candidate>,
    reason: Option<String>,
    revision: u64,
    generation: u64,
    binding_epoch: u64,
}

impl TargetState {
    pub fn validate_routing(&mut self, query: &impl crate::routing::Query) -> bool {
        let changed = self.routing.validate(self.bound_identity(), query);
        if changed {
            self.revision += 1;
        }
        changed
    }
    pub fn discover_routing(&mut self, query: &impl crate::routing::Query) {
        if let Some(root) = self.bound_identity() {
            self.routing.bind(root, query);
        } else {
            self.routing.reset();
        }
        self.revision += 1;
    }
    pub fn choose_receiver(
        &mut self,
        token: &str,
        query: &impl crate::routing::Query,
    ) -> Result<BindingStatus, String> {
        self.routing.choose(self.bound_identity(), token, query)?;
        self.revision += 1;
        Ok(self.status())
    }
    pub fn bound_identity(&self) -> Option<(Identity, u64)> {
        self.bound
            .as_ref()
            .map(|bound| (bound.identity.clone(), self.binding_epoch))
    }
    pub fn status(&self) -> BindingStatus {
        BindingStatus {
            revision: self.revision,
            epoch: self.binding_epoch,
            bound: self.bound.as_ref().map(|bound| bound.view.clone()),
            reason: self.reason.clone(),
            routing: self.routing.status(),
        }
    }

    pub fn refresh(
        &mut self,
        query: &impl WindowQuery,
        include_hidden: bool,
    ) -> Result<TargetList, String> {
        self.validate(query);
        // Even a failed refresh invalidates stale list tokens, while a valid binding survives.
        self.candidates.clear();
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or("候选序号已耗尽，请重启程序。")?;
        let result = query
            .enumerate(include_hidden)
            .map_err(|_| "无法刷新窗口列表，请重试；不会自动提升权限。")?;
        for (index, observed) in result.windows.into_iter().enumerate() {
            let view = TargetView {
                token: format!("{}:{}", self.generation, index),
                title: observed.title,
                exe: observed.exe,
                pid: observed.identity.pid,
                status: observed.status,
            };
            self.candidates.push(Candidate {
                view,
                identity: observed.identity,
            });
        }
        Ok(TargetList {
            windows: self
                .candidates
                .iter()
                .map(|candidate| candidate.view.clone())
                .collect(),
            inaccessible: result.inaccessible,
            binding: self.status(),
        })
    }

    pub fn bind(&mut self, token: &str, query: &impl WindowQuery) -> Result<BindingStatus, String> {
        self.validate(query);
        let candidate = self
            .candidates
            .iter()
            .find(|candidate| candidate.view.token == token)
            .cloned()
            .ok_or("候选已过期，请刷新后重新选择。")?;
        let current = query
            .inspect(candidate.identity.hwnd)
            .map_err(|error| match error {
                QueryError::Permission => "无法验证目标进程权限，请刷新后选择可读取的窗口。",
                _ => "目标窗口已失效，请刷新后重新选择。",
            })?;
        if current.identity != candidate.identity {
            return Err("目标窗口或进程实例已变化，请刷新后重新选择。".into());
        }
        let mut bound = candidate;
        bound.view.title = current.title;
        bound.view.exe = current.exe;
        bound.view.status = current.status;
        self.bound = Some(bound);
        self.binding_epoch += 1;
        self.reason = None;
        self.revision += 1;
        Ok(self.status())
    }

    pub fn unbind(&mut self) -> BindingStatus {
        self.routing.reset();
        self.bound = None;
        self.binding_epoch += 1;
        self.reason = None;
        self.revision += 1;
        self.status()
    }

    pub fn validate(&mut self, query: &impl WindowQuery) -> bool {
        let Some(bound) = &self.bound else {
            return false;
        };
        match query.inspect(bound.identity.hwnd) {
            Ok(current) if current.identity == bound.identity => {
                if current.title != bound.view.title
                    || current.exe != bound.view.exe
                    || current.status != bound.view.status
                {
                    let bound = self.bound.as_mut().unwrap();
                    bound.view.title = current.title;
                    bound.view.exe = current.exe;
                    bound.view.status = current.status;
                    self.revision += 1;
                    return true;
                }
                false
            }
            result => {
                self.bound = None;
                self.binding_epoch += 1;
                self.reason = Some(
                    match result {
                        Err(QueryError::Permission) => {
                            "无法继续验证目标进程权限，已解除绑定。请重新选择和绑定。"
                        }
                        Err(QueryError::Gone) => "目标窗口已关闭或进程已退出，已解除绑定。",
                        _ => "目标身份已变化或无法验证，已解除绑定；不会自动跟随重启实例。",
                    }
                    .into(),
                );
                self.revision += 1;
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct Fake {
        windows: RefCell<Vec<ObservedWindow>>,
        fail: RefCell<Option<QueryError>>,
        omit_from_list: RefCell<bool>,
        fail_enumeration: RefCell<bool>,
    }
    impl WindowQuery for Fake {
        fn enumerate(&self, include_hidden: bool) -> Result<Enumeration, QueryError> {
            if *self.fail_enumeration.borrow() {
                return Err(QueryError::Failed);
            }
            if let Some(error) = self.fail.borrow().clone() {
                return Err(error);
            }
            Ok(Enumeration {
                windows: self
                    .windows
                    .borrow()
                    .iter()
                    .filter(|window| {
                        !*self.omit_from_list.borrow()
                            && (include_hidden
                                || (window.status != WindowStatus::Hidden
                                    && !window.title.is_empty()))
                    })
                    .cloned()
                    .collect(),
                inaccessible: 2,
            })
        }
        fn inspect(&self, hwnd: usize) -> Result<ObservedWindow, QueryError> {
            if let Some(error) = self.fail.borrow().clone() {
                return Err(error);
            }
            self.windows
                .borrow()
                .iter()
                .find(|window| window.identity.hwnd == hwnd)
                .cloned()
                .ok_or(QueryError::Gone)
        }
    }
    fn window(hwnd: usize, pid: u32, created: u64) -> ObservedWindow {
        ObservedWindow {
            identity: Identity {
                hwnd,
                pid,
                created,
                thread: 10,
            },
            title: "Fixture window".into(),
            exe: "Fixture.exe".into(),
            status: WindowStatus::Visible,
        }
    }
    fn fixture() -> Fake {
        Fake {
            windows: RefCell::new(vec![window(1, 20, 30)]),
            fail: RefCell::new(None),
            omit_from_list: RefCell::new(false),
            fail_enumeration: RefCell::new(false),
        }
    }
    #[test]
    fn selection_is_not_binding_and_same_name_windows_are_not_merged() {
        let query = fixture();
        query
            .windows
            .borrow_mut()
            .extend([window(2, 20, 30), window(3, 21, 40)]);
        let mut state = TargetState::default();
        let list = state.refresh(&query, false).unwrap();
        assert_eq!(list.windows.len(), 3);
        assert_eq!(list.inaccessible, 2);
        assert!(state.status().bound.is_none());
        assert_ne!(list.windows[0].token, list.windows[1].token);
        state.bind(&list.windows[1].token, &query).unwrap();
        assert_eq!(state.bound.as_ref().unwrap().identity.hwnd, 2);
    }
    #[test]
    fn stale_token_and_forged_token_cannot_bind() {
        let query = fixture();
        let mut state = TargetState::default();
        let old = state.refresh(&query, false).unwrap().windows[0]
            .token
            .clone();
        state.refresh(&query, false).unwrap();
        assert!(state.bind(&old, &query).is_err());
        assert!(state.bind("invented", &query).is_err());
        assert!(state.bound.is_none());
    }

    #[test]
    fn empty_list_and_bind_permission_failure_are_honest() {
        let query = fixture();
        let mut state = TargetState::default();
        let token = state.refresh(&query, false).unwrap().windows[0]
            .token
            .clone();
        *query.fail.borrow_mut() = Some(QueryError::Permission);
        assert!(state.bind(&token, &query).is_err());
        assert!(state.bound.is_none());
        *query.fail.borrow_mut() = None;
        query.windows.borrow_mut().clear();
        assert!(state.refresh(&query, false).unwrap().windows.is_empty());
    }
    #[test]
    fn revalidate_rejects_pid_creation_time_and_thread_changes() {
        for change in 0..3 {
            let query = fixture();
            let mut state = TargetState::default();
            let token = state.refresh(&query, false).unwrap().windows[0]
                .token
                .clone();
            let mut windows = query.windows.borrow_mut();
            let id = &mut windows[0].identity;
            match change {
                0 => id.pid += 1,
                1 => id.created += 1,
                _ => id.thread += 1,
            };
            drop(windows);
            assert!(state.bind(&token, &query).is_err());
            assert!(state.bound.is_none());
        }
    }
    #[test]
    fn refresh_preserves_binding_but_exit_and_restart_never_autobind() {
        let query = fixture();
        let mut state = TargetState::default();
        let token = state.refresh(&query, false).unwrap().windows[0]
            .token
            .clone();
        state.bind(&token, &query).unwrap();
        state.refresh(&query, false).unwrap();
        assert!(state.bound.is_some());
        query.windows.borrow_mut().clear();
        assert!(state.validate(&query));
        assert!(state.bound.is_none());
        assert!(state.reason.is_some());
        query.windows.borrow_mut().push(window(1, 20, 31));
        state.refresh(&query, false).unwrap();
        assert!(state.bound.is_none());
    }
    #[test]
    fn permission_or_failed_refresh_cannot_keep_an_unverifiable_binding() {
        let query = fixture();
        let mut state = TargetState::default();
        let token = state.refresh(&query, false).unwrap().windows[0]
            .token
            .clone();
        state.bind(&token, &query).unwrap();
        *query.fail.borrow_mut() = Some(QueryError::Permission);
        assert!(state.validate(&query));
        assert!(state.bound.is_none());
        assert!(state.refresh(&query, false).is_err());
        assert!(state.candidates.is_empty());
        *query.fail.borrow_mut() = None;
        assert!(state.bind(&token, &query).is_err());
    }
    #[test]
    fn periodic_check_detects_instance_replacement_and_unbind_is_explicit() {
        let query = fixture();
        let mut state = TargetState::default();
        let token = state.refresh(&query, false).unwrap().windows[0]
            .token
            .clone();
        state.bind(&token, &query).unwrap();
        query.windows.borrow_mut()[0].identity.created += 1;
        assert!(state.validate(&query));
        assert!(state.bound.is_none());
        let token = state.refresh(&query, false).unwrap().windows[0]
            .token
            .clone();
        state.bind(&token, &query).unwrap();
        state.unbind();
        assert!(state.bound.is_none());
        assert!(state.reason.is_none());
    }

    #[test]
    fn hidden_untitled_binding_survives_range_change_and_refresh_absence() {
        let query = fixture();
        let mut state = TargetState::default();
        let token = state.refresh(&query, false).unwrap().windows[0]
            .token
            .clone();
        state.bind(&token, &query).unwrap();
        {
            let mut windows = query.windows.borrow_mut();
            windows[0].title.clear();
            windows[0].status = WindowStatus::Hidden;
        }
        assert!(state.validate(&query));
        let bound = state.status().bound.unwrap();
        assert_eq!(bound.status, WindowStatus::Hidden);
        assert!(bound.title.is_empty());
        let expanded = state.refresh(&query, true).unwrap();
        assert_eq!(expanded.windows.len(), 1);
        assert!(expanded.windows[0].title.is_empty());
        assert_eq!(expanded.windows[0].status, WindowStatus::Hidden);
        let old_token = expanded.windows[0].token.clone();
        let narrow = state.refresh(&query, false).unwrap();
        assert!(narrow.windows.is_empty());
        assert!(narrow.binding.bound.is_some());
        assert!(state.bind(&old_token, &query).is_err());
        *query.omit_from_list.borrow_mut() = true;
        assert!(state.refresh(&query, true).unwrap().windows.is_empty());
        assert!(state.status().bound.is_some());
        assert_eq!(
            state.bound.as_ref().unwrap().identity,
            query.windows.borrow()[0].identity
        );
        query.windows.borrow_mut().clear();
        assert!(state.validate(&query));
        assert!(state.status().bound.is_none());
    }

    #[test]
    fn visible_minimized_and_hidden_candidates_use_requested_range() {
        let query = fixture();
        let mut minimized = window(2, 21, 31);
        minimized.status = WindowStatus::Minimized;
        let mut hidden = window(3, 22, 32);
        hidden.status = WindowStatus::Hidden;
        let mut untitled = window(4, 23, 33);
        untitled.title.clear();
        query
            .windows
            .borrow_mut()
            .extend([minimized, hidden, untitled]);
        let mut state = TargetState::default();
        let narrow = state.refresh(&query, false).unwrap();
        assert_eq!(narrow.windows.len(), 2);
        assert_eq!(narrow.windows[1].status, WindowStatus::Minimized);
        let expanded = state.refresh(&query, true).unwrap();
        assert_eq!(expanded.windows.len(), 4);
        state.bind(&expanded.windows[2].token, &query).unwrap();
        assert_eq!(state.status().bound.unwrap().status, WindowStatus::Hidden);
        state.bind(&expanded.windows[3].token, &query).unwrap();
        assert!(state.status().bound.unwrap().title.is_empty());
    }

    #[test]
    fn enumeration_failure_expires_candidates_but_keeps_live_binding() {
        let query = fixture();
        let mut state = TargetState::default();
        let token = state.refresh(&query, false).unwrap().windows[0]
            .token
            .clone();
        state.bind(&token, &query).unwrap();
        *query.fail_enumeration.borrow_mut() = true;
        assert!(state.refresh(&query, true).is_err());
        assert!(state.candidates.is_empty());
        assert!(state.status().bound.is_some());
        assert!(state.bind(&token, &query).is_err());
    }

    #[test]
    fn preview_epoch_tracks_rebinding_not_display_or_candidate_refresh() {
        let query = fixture();
        let mut state = TargetState::default();
        let token = state.refresh(&query, false).unwrap().windows[0]
            .token
            .clone();
        state.bind(&token, &query).unwrap();
        let binding = state.bound_identity().unwrap();
        query.windows.borrow_mut()[0].title = "Renamed fixture".into();
        state.validate(&query);
        assert_eq!(state.bound_identity().unwrap(), binding);
        let list = state.refresh(&query, false).unwrap();
        assert_eq!(state.bound_identity().unwrap(), binding);
        state.bind(&list.windows[0].token, &query).unwrap();
        assert_eq!(state.bound_identity().unwrap().0, binding.0);
        assert_ne!(state.bound_identity().unwrap().1, binding.1);
        state.unbind();
        assert!(state.bound_identity().is_none());
        assert!(state.status().epoch > binding.1);
    }
}
