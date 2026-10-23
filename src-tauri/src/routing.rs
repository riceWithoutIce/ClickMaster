//! Backend-owned receiver choices. Revalidation never discovers a replacement.
use crate::targets::Identity;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receiver {
    pub identity: Identity,
    pub exe: String,
    pub class: String,
    pub width: i32,
    pub height: i32,
}
pub struct Discovery {
    pub root: Receiver,
    pub hosted: bool,
    pub contents: Vec<Receiver>,
}
pub trait Query {
    fn discover(&self, root: &Identity) -> Result<Discovery, ()>;
    fn inspect(&self, root: &Identity, receiver: &Identity) -> Result<Receiver, ()>;
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub token: String,
    pub exe: String,
    pub pid: u32,
    pub class: String,
    pub width: i32,
    pub height: i32,
    pub is_self: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub root: (Identity, u64),
    pub receiver: Identity,
    pub epoch: u64,
    pub view: View,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub epoch: u64,
    pub phase: String,
    pub message: String,
    pub selected: Option<View>,
    pub candidates: Vec<View>,
}
#[derive(Clone)]
struct Choice {
    receiver: Receiver,
    view: View,
}
#[derive(Default)]
pub struct State {
    root: Option<(Identity, u64)>,
    epoch: u64,
    phase: String,
    message: String,
    candidates: Vec<Choice>,
    selected: Option<Choice>,
}
impl State {
    pub fn status(&self) -> Status {
        Status {
            epoch: self.epoch,
            phase: if self.phase.is_empty() {
                "unbound".into()
            } else {
                self.phase.clone()
            },
            message: self.message.clone(),
            selected: self.selected.as_ref().map(|item| item.view.clone()),
            candidates: self
                .candidates
                .iter()
                .map(|item| item.view.clone())
                .collect(),
        }
    }
    pub fn snapshot(&self) -> Option<Snapshot> {
        Some(Snapshot {
            root: self.root.clone()?,
            receiver: self.selected.as_ref()?.receiver.identity.clone(),
            epoch: self.epoch,
            view: self.selected.as_ref()?.view.clone(),
        })
    }
    pub fn reset(&mut self) {
        self.epoch += 1;
        self.root = None;
        self.selected = None;
        self.candidates.clear();
        self.phase = "unbound".into();
        self.message.clear();
    }
    fn invalid(&mut self, message: &str) {
        self.epoch += 1;
        self.selected = None;
        self.candidates.clear();
        self.phase = "invalid".into();
        self.message = message.into();
    }
    pub fn bind(&mut self, root: (Identity, u64), query: &impl Query) {
        self.reset();
        self.root = Some(root.clone());
        let Ok(found) = query.discover(&root.0) else {
            self.invalid("无法验证输入接收窗口，请重新绑定；不会回退到宿主。");
            return;
        };
        if found.root.identity != root.0
            || found.contents.iter().any(|item| item.identity == root.0)
        {
            self.invalid("接收窗口结构无法确认，请重新绑定。");
            return;
        }
        let content_count = found.contents.len();
        let mut receivers = vec![found.root];
        receivers.extend(found.contents);
        for (index, receiver) in receivers.into_iter().enumerate() {
            let view = View {
                token: format!("r{}:{index}", self.epoch),
                exe: receiver.exe.clone(),
                pid: receiver.identity.pid,
                class: receiver.class.clone(),
                width: receiver.width,
                height: receiver.height,
                is_self: receiver.identity == root.0,
            };
            self.candidates.push(Choice { receiver, view });
        }
        if !found.hosted {
            self.selected = self.candidates.first().cloned();
            self.phase = "self".into();
            self.message = "输入发送到绑定窗口自身。".into();
        } else if content_count == 1 {
            self.selected = self.candidates.get(1).cloned();
            self.phase = "automatic".into();
            self.message = "已自动选择唯一内容窗口；不代表目标一定响应。".into();
        } else {
            self.phase = "choose".into();
            self.message = if content_count == 0 {
                "未发现可用内容窗口；请重新绑定，或明确选择绑定窗口自身。"
            } else {
                "存在多个内容窗口，请明确选择输入接收窗口。"
            }
            .into();
        }
    }
    pub fn validate(&mut self, root: Option<(Identity, u64)>, query: &impl Query) -> bool {
        if self.root.is_none() {
            return false;
        }
        if root != self.root {
            self.invalid("绑定窗口已变化，输入路径失效，请重新绑定。");
            self.root = None;
            return true;
        }
        if let Some(selected) = &self.selected {
            let valid = query
                .inspect(&self.root.as_ref().unwrap().0, &selected.receiver.identity)
                .is_ok_and(|current| {
                    current.identity == selected.receiver.identity
                        && current.class == selected.receiver.class
                });
            if !valid {
                self.invalid("原输入接收窗口已失效或归属变化，请重新绑定；不会跟随替代窗口。");
                return true;
            }
        }
        false
    }
    pub fn choose(
        &mut self,
        root: Option<(Identity, u64)>,
        token: &str,
        query: &impl Query,
    ) -> Result<(), String> {
        self.validate(root, query);
        if self.root.is_none() || self.phase == "invalid" {
            return Err("接收路径已失效，请重新绑定。".into());
        }
        let choice = self
            .candidates
            .iter()
            .find(|item| item.view.token == token)
            .cloned()
            .ok_or("接收候选已过期，请重新绑定后选择。")?;
        let current = query
            .inspect(&self.root.as_ref().unwrap().0, &choice.receiver.identity)
            .map_err(|_| "接收候选已失效，请重新绑定。")?;
        if current.identity != choice.receiver.identity || current.class != choice.receiver.class {
            return Err("接收候选身份已变化，请重新绑定。".into());
        }
        self.epoch += 1;
        // Tokens are scoped to the original discovery; switching does not discover new windows.
        self.selected = Some(choice);
        self.phase = "manual".into();
        self.message = "已明确选择输入接收窗口。".into();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct Fake {
        hosted: bool,
        children: RefCell<Vec<Receiver>>,
        fail: RefCell<bool>,
        relation: RefCell<bool>,
    }
    fn receiver(hwnd: usize) -> Receiver {
        Receiver {
            identity: Identity {
                hwnd,
                pid: 10 + hwnd as u32,
                created: 20,
                thread: 30,
            },
            exe: "Fixture.exe".into(),
            class: if hwnd == 1 {
                "ApplicationFrameWindow"
            } else {
                "Windows.UI.Core.CoreWindow"
            }
            .into(),
            width: 500,
            height: 400,
        }
    }
    fn root() -> (Identity, u64) {
        (receiver(1).identity, 4)
    }
    impl Query for Fake {
        fn discover(&self, _: &Identity) -> Result<Discovery, ()> {
            if *self.fail.borrow() {
                return Err(());
            }
            Ok(Discovery {
                root: receiver(1),
                hosted: self.hosted,
                contents: self.children.borrow().clone(),
            })
        }
        fn inspect(&self, _: &Identity, expected: &Identity) -> Result<Receiver, ()> {
            if *self.fail.borrow() || !*self.relation.borrow() {
                return Err(());
            }
            if expected.hwnd == 1 {
                return Ok(receiver(1));
            }
            self.children
                .borrow()
                .iter()
                .find(|item| item.identity.hwnd == expected.hwnd)
                .cloned()
                .ok_or(())
        }
    }
    fn fake(hosted: bool, count: usize) -> Fake {
        Fake {
            hosted,
            children: RefCell::new((2..2 + count).map(receiver).collect()),
            fail: RefCell::new(false),
            relation: RefCell::new(true),
        }
    }
    #[test]
    fn ordinary_self_and_unique_hosted_are_distinct() {
        for (hosted, count, expected) in [(false, 0, 1), (true, 1, 2)] {
            let mut state = State::default();
            state.bind(root(), &fake(hosted, count));
            assert_eq!(state.snapshot().unwrap().receiver.hwnd, expected);
        }
    }
    #[test]
    fn ambiguity_needs_explicit_token_and_tokens_do_not_cross_binding() {
        let query = fake(true, 2);
        let mut state = State::default();
        state.bind(root(), &query);
        assert!(state.snapshot().is_none());
        assert_eq!(state.status().phase, "choose");
        let token = state.status().candidates[2].token.clone();
        assert!(state.choose(Some(root()), "forged", &query).is_err());
        state.choose(Some(root()), &token, &query).unwrap();
        assert_eq!(state.snapshot().unwrap().receiver.hwnd, 3);
        state.bind(root(), &query);
        assert!(state.choose(Some(root()), &token, &query).is_err());
    }
    #[test]
    fn invalid_child_does_not_follow_replacement_or_fall_back() {
        for change in 0..5 {
            let query = fake(true, 1);
            let mut state = State::default();
            state.bind(root(), &query);
            match change {
                0 => query.children.borrow_mut()[0].identity.created += 1,
                1 => query.children.borrow_mut()[0].identity.thread += 1,
                2 => *query.relation.borrow_mut() = false,
                3 => query.children.borrow_mut()[0].identity.pid += 1,
                _ => query.children.borrow_mut().clear(),
            }
            assert!(state.validate(Some(root()), &query));
            assert!(state.snapshot().is_none());
            query.children.borrow_mut().clear();
            query.children.borrow_mut().push(receiver(2));
            *query.relation.borrow_mut() = true;
            assert!(!state.validate(Some(root()), &query));
            assert!(state.snapshot().is_none());
            assert!(state.choose(Some(root()), "r1:0", &query).is_err());
        }
    }
    #[test]
    fn root_epoch_change_or_query_failure_never_falls_back() {
        let query = fake(true, 1);
        let mut state = State::default();
        state.bind(root(), &query);
        assert!(state.validate(Some((root().0, 9)), &query));
        assert!(state.snapshot().is_none());
        *query.fail.borrow_mut() = true;
        state.bind(root(), &query);
        assert!(state.snapshot().is_none());
        assert_eq!(state.status().phase, "invalid");
    }
    #[test]
    fn validation_keeps_snapshot_and_switch_changes_route_epoch() {
        let query = fake(true, 1);
        let mut state = State::default();
        state.bind(root(), &query);
        let before = state.snapshot().unwrap();
        assert!(!state.validate(Some(root()), &query));
        assert_eq!(state.snapshot(), Some(before.clone()));
        let own = state.status().candidates[0].token.clone();
        state.choose(Some(root()), &own, &query).unwrap();
        assert_ne!(state.snapshot().unwrap().epoch, before.epoch);
        assert_eq!(state.snapshot().unwrap().receiver.hwnd, 1);
    }
}
