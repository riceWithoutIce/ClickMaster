//! One owned ephemeral slot per active task; no retained frame/session history.
use crate::ipc_worker;
use std::{
    fs,
    path::{Path, PathBuf},
};
pub struct Slot {
    root: PathBuf,
    dir: Option<PathBuf>,
}
impl Slot {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            dir: None,
        }
    }
    pub fn prepare(&mut self) -> Result<(PathBuf, String), String> {
        // The caller must have observed its directly owned worker exit before reusing.
        if let Some(dir) = &self.dir {
            clear(dir)?;
            ipc_worker::renew(dir).map(|token| (dir.clone(), token))
        } else {
            let (dir, token) = ipc_worker::attempt(&self.root)?;
            self.dir = Some(dir.clone());
            fs::create_dir(dir.join("tmp")).map_err(|_| "无法创建本轮临时目录。")?;
            Ok((dir, token))
        }
    }
    pub fn finish(&mut self) -> Result<(), String> {
        if let Some(dir) = &self.dir {
            clear(dir)?;
            match fs::remove_dir(dir.join("tmp")) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err("本轮临时目录仍被占用，清理未确认。".into()),
            }
            fs::remove_dir(dir).map_err(|_| "无法清理本轮独立目录。")?;
            self.dir = None;
        }
        Ok(())
    }
}
fn clear(dir: &Path) -> Result<(), String> {
    for name in [
        "authorization.json",
        "claimed.json",
        "status.json",
        "status.next",
    ] {
        match fs::remove_file(dir.join(name)) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("无法清理本轮 IPC 状态，禁止继续。".into()),
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_frames_use_one_slot_and_cleanup_never_touches_existing_data() {
        let root =
            PathBuf::from(std::env::var_os("CLICKMASTER_TEST_ROOT").unwrap()).join("bounded-slot");
        fs::create_dir_all(root.join("ipc/old-evidence")).unwrap();
        fs::write(root.join("ipc/old-evidence/keep"), "keep").unwrap();
        let mut slot = Slot::new(&root);
        let mut original = None;
        for _ in 0..20 {
            let (dir, _) = slot.prepare().unwrap();
            if let Some(old) = &original {
                assert_eq!(&dir, old);
            } else {
                original = Some(dir.clone());
            }
            fs::write(dir.join("claimed.json"), "true").unwrap();
            assert_eq!(fs::read_dir(root.join("ipc")).unwrap().count(), 2);
        }
        slot.finish().unwrap();
        assert_eq!(
            fs::read_to_string(root.join("ipc/old-evidence/keep")).unwrap(),
            "keep"
        );
        assert_eq!(fs::read_dir(root.join("ipc")).unwrap().count(), 1);
    }
}
