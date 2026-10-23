use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub minimize_keep_taskbar: bool,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

pub struct SettingsStore {
    pub settings: Settings,
    pub warning: Option<String>,
    path: PathBuf,
    invalid: bool,
}

impl SettingsStore {
    pub fn load(root: &Path) -> Self {
        let path = root.join("config/settings.json");
        let mut result = Self {
            settings: Settings::default(),
            warning: None,
            path,
            invalid: false,
        };
        match fs::read(&result.path) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(settings) => result.settings = settings,
                Err(_) => {
                    result.invalid = true;
                    result.warning = Some("配置格式无效，已使用默认偏好；原文件已保留。请修复或移走配置文件后重新启动。".into());
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {
                result.invalid = true;
                result.warning =
                    Some("无法读取配置，已使用默认偏好；请检查配置文件权限后重新启动。".into());
            }
        }
        result
    }

    pub fn save_preference(&mut self, keep: bool) -> Result<(), String> {
        if self.invalid {
            return Err(self
                .warning
                .clone()
                .unwrap_or_else(|| "配置不可用，原文件已保留。".into()));
        }
        let mut next = self.settings.clone();
        next.minimize_keep_taskbar = keep;
        let bytes = serde_json::to_vec_pretty(&next).map_err(|_| "无法序列化配置。")?;
        let temp = self.path.with_extension("json.pending");
        let save = || -> std::io::Result<()> {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            atomic_replace(&temp, &self.path)
        };
        // Only remove a temporary file if this attempt created it.
        if temp.exists() {
            return Err("存在未完成的配置保存文件，请检查后重试；原配置未改变。".into());
        }
        if save().is_err() {
            let _ = fs::remove_file(&temp);
            return Err("保存失败，请检查数据目录权限；原配置和上次偏好已保留。".into());
        }
        self.settings = next;
        Ok(())
    }
}

#[cfg(windows)]
fn atomic_replace(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };
    let source: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let target: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe {
        MoveFileExW(
            source.as_ptr(),
            target.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    } == 0
    {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(name: &str) -> PathBuf {
        let root = PathBuf::from(
            std::env::var_os("CLICKMASTER_TEST_ROOT").expect("project test runner required"),
        )
        .join("settings")
        .join(name);
        fs::create_dir_all(root.join("config")).unwrap();
        root
    }
    #[test]
    fn roundtrip_and_preserve_unknown_fields() {
        let root = fixture("roundtrip");
        fs::write(
            root.join("config/settings.json"),
            br#"{"future_option":42}"#,
        )
        .unwrap();
        let mut store = SettingsStore::load(&root);
        assert!(!store.settings.minimize_keep_taskbar);
        store.save_preference(true).unwrap();
        let restored = SettingsStore::load(&root);
        assert!(restored.settings.minimize_keep_taskbar);
        assert_eq!(restored.settings.extra["future_option"], 42);
    }
    #[test]
    fn invalid_file_is_never_overwritten() {
        let root = fixture("invalid");
        let path = root.join("config/settings.json");
        let invalid = b"{ invalid";
        fs::write(&path, invalid).unwrap();
        let mut store = SettingsStore::load(&root);
        assert!(store.warning.is_some());
        assert!(store.save_preference(true).is_err());
        assert_eq!(fs::read(path).unwrap(), invalid);
        assert!(!store.settings.minimize_keep_taskbar);
    }
    #[test]
    fn failed_save_preserves_file_and_active_preference() {
        let root = fixture("failure");
        let path = root.join("config/settings.json");
        fs::write(&path, b"{\"minimize_keep_taskbar\":false}").unwrap();
        let original = fs::read(&path).unwrap();
        let mut store = SettingsStore::load(&root);
        // A held file without delete sharing prevents atomic replacement on Windows.
        use std::os::windows::fs::OpenOptionsExt;
        let held = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&path)
            .unwrap();
        assert!(store.save_preference(true).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        assert!(!store.settings.minimize_keep_taskbar);
        assert!(!root.join("config/settings.json.pending").exists());
        drop(held);
        store.save_preference(true).unwrap();
        assert!(SettingsStore::load(&root).settings.minimize_keep_taskbar);
    }
}
