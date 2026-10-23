//! Same executable worker: fail closed before any UI or game transport.
use crate::{
    ipc::{self, Report},
    ipc_native::{self as native, Lease},
    targets::Identity,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authorization {
    pub token: String,
    pub target: Identity,
    pub parent: u32,
    pub parent_created: u64,
    #[serde(default)]
    pub diagnostics: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    token_hash: String,
    binary_hash: String,
}
pub fn valid_run(run: &str) -> bool {
    run.len() == 64
        && run
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase())
}
pub fn attempt(root: &Path) -> Result<(PathBuf, String), String> {
    let run = native::random()?;
    let dir = root.join("ipc").join(run);
    fs::create_dir_all(dir.parent().ok_or("无法准备 IPC 数据。")?)
        .map_err(|_| "无法准备 IPC 数据。")?;
    fs::create_dir(&dir).map_err(|_| "无法准备独立 IPC 数据目录。")?;
    let token = match renew(&dir) {
        Ok(token) => token,
        Err(error) => {
            let _ = fs::remove_file(dir.join("authorization.json"));
            let _ = fs::remove_dir(&dir);
            return Err(error);
        }
    };
    Ok((dir, token))
}
pub(crate) fn renew(dir: &Path) -> Result<String, String> {
    let token = native::random()?;
    let manifest = Manifest {
        token_hash: native::sha(token.as_bytes())?,
        binary_hash: {
            let _measure = crate::performance::span(crate::performance::Stage::ParentExeHash);
            native::file_sha(&std::env::current_exe().map_err(|_| "无法核验程序。")?)?
        },
    };
    create(&dir.join("authorization.json"), &manifest)?;
    Ok(token)
}
fn create(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|_| "无法创建 IPC 任务记录。")?;
    let bytes = serde_json::to_vec(value).map_err(|_| "无法编码 IPC 任务记录。")?;
    file.write_all(&bytes)
        .map_err(|_| "无法写入 IPC 任务记录。".into())
}
pub fn report(dir: &Path, value: &Report) -> Result<(), String> {
    fs::write(
        dir.join("status.next"),
        serde_json::to_vec(value).map_err(|_| "无法编码状态。")?,
    )
    .map_err(|_| "无法记录 IPC 状态。")?;
    fs::rename(dir.join("status.next"), dir.join("status.json"))
        .map_err(|_| "无法发布 IPC 状态。".into())
}
pub(crate) fn authorize(dir: &Path, auth: &Authorization) -> Result<native::Handle, String> {
    let _measure = crate::performance::span(crate::performance::Stage::Authorization);
    let bytes = fs::read(dir.join("authorization.json")).map_err(|_| "授权缺失。")?;
    if bytes.len() > 1024 {
        return Err("授权无效。".into());
    }
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|_| "授权无效。")?;
    let exe = std::env::current_exe().map_err(|_| "程序位置无效。")?;
    if auth.token.len() != 64 || native::sha(auth.token.as_bytes())? != manifest.token_hash || {
        let _measure = crate::performance::span(crate::performance::Stage::WorkerExeHash);
        native::file_sha(&exe)?
    }
        != manifest.binary_hash
    {
        return Err("授权不匹配。".into());
    }
    // Inherited anonymous stdin is the private parent-to-child authorization channel.
    use windows_sys::Win32::System::{
        Console::{GetStdHandle, STD_INPUT_HANDLE},
        Pipes::GetNamedPipeServerProcessId,
    };
    let mut server = 0;
    if unsafe { GetNamedPipeServerProcessId(GetStdHandle(STD_INPUT_HANDLE), &mut server) } == 0
        || server != auth.parent
    {
        return Err("授权来源不匹配。".into());
    }
    let (parent, created, parent_exe) = native::process(auth.parent)?;
    if created != auth.parent_created || parent_exe != exe || !native::running(&parent) {
        return Err("原父进程已变化。".into());
    }
    create(&dir.join("claimed.json"), &true)?;
    Ok(parent)
}
/// Exactly one write; transport-level read observation can never request a retry.
pub(crate) fn send_frame(
    pipe: &native::Handle,
    check: impl FnOnce() -> Result<(), ipc::Failure>,
    mut publish: impl FnMut(&str, ipc::Receipt) -> Result<(), String>,
) -> Result<(), ipc::Failure> {
    send_frame_with_drain(pipe, check, &mut publish, |pipe| native::drain(pipe))
}
pub(crate) fn send_frame_with_drain(
    pipe: &native::Handle,
    check: impl FnOnce() -> Result<(), ipc::Failure>,
    mut publish: impl FnMut(&str, ipc::Receipt) -> Result<(), String>,
    drain: impl FnOnce(&native::Handle) -> Result<(), u32>,
) -> Result<(), ipc::Failure> {
    let mut receipt = ipc::Receipt {
        connected: true,
        ..Default::default()
    };
    let status_error =
        |_| ipc::Failure::new(ipc::FailureStage::Status, ipc::FailureReason::ReportInvalid);
    publish("writing", receipt.clone()).map_err(status_error)?;
    check()?;
    native::write(pipe)?;
    receipt.written = true;
    publish("draining", receipt.clone()).map_err(status_error)?;
    match drain(pipe) {
        Ok(()) => receipt.read_status = ipc::ReadStatus::Confirmed,
        Err(code) => {
            receipt.read_status = ipc::ReadStatus::Failed;
            receipt.read_error_code = Some(code);
        }
    }
    publish("completed", receipt).map_err(status_error)
}

/// Persistent transport deliberately does not call FlushFileBuffers after a
/// frame. A receiver may keep the connection open without acknowledging each
/// frame; the write itself is the delivery boundary and read status remains a
/// diagnostic. This keeps one connection usable for the next command.
pub(crate) fn send_persistent_frame(
    pipe: &native::Handle,
    check: impl FnOnce() -> Result<(), ipc::Failure>,
    mut publish: impl FnMut(&str, ipc::Receipt) -> Result<(), String>,
) -> Result<(), ipc::Failure> {
    let mut receipt = ipc::Receipt {
        connected: true,
        ..Default::default()
    };
    publish("writing", receipt.clone()).map_err(|_| {
        ipc::Failure::new(ipc::FailureStage::Status, ipc::FailureReason::ReportInvalid)
    })?;
    check()?;
    native::write(pipe)?;
    receipt.written = true;
    receipt.read_status = ipc::ReadStatus::Unconfirmed;
    publish("completed", receipt).map_err(|_| {
        ipc::Failure::new(ipc::FailureStage::Status, ipc::FailureReason::ReportInvalid)
    })
}
pub fn entry() -> i32 {
    if std::env::var("CLICKMASTER_IPC_PERSISTENT").ok().as_deref() == Some("1") {
        return persistent_entry();
    }
    let result = (|| -> Result<(), String> {
        let run = std::env::var("CLICKMASTER_IPC_RUN").map_err(|_| "授权缺失。")?;
        std::env::remove_var("CLICKMASTER_IPC_RUN");
        if !valid_run(&run) {
            return Err("授权格式无效。".into());
        }
        let exe = std::env::current_exe().map_err(|_| "程序位置无效。")?;
        let dir = crate::data_root(&exe)?.join("ipc").join(run);
        // No argument path, current-directory fallback, target identity or secret is persisted.
        if !dir.join("authorization.json").is_file() {
            return Err("授权缺失。".into());
        }
        let mut buffer = Vec::new();
        std::io::stdin()
            .take(4097)
            .read_to_end(&mut buffer)
            .map_err(|_| "无法读取授权。")?;
        if buffer.len() > 4096 {
            return Err("授权过大。".into());
        }
        let auth: Authorization = serde_json::from_slice(&buffer).map_err(|_| "授权无效。")?;
        crate::performance::set_worker(auth.diagnostics != 0);
        let parent = authorize(&dir, &auth)?;
        let origin = Instant::now();
        // Even if the UI process dies unexpectedly, this worker cannot remain a pipe server.
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(50));
            if !native::running(&parent) || origin.elapsed().as_millis() >= ipc::TOTAL_MS as u128 {
                std::process::exit(70);
            }
        });
        let mut current = Report::new("checking");
        current.performance_epoch = auth.diagnostics;
        let mut stage = Instant::now();
        let mut publish = |phase: &str, receipt: ipc::Receipt| -> Result<(), String> {
            if current.phase != phase {
                stage = Instant::now();
            }
            current.phase = phase.into();
            current.elapsed_ms = origin.elapsed().as_millis() as u64;
            current.stage_ms = stage.elapsed().as_millis() as u64;
            current.receipt = receipt;
            current.performance = crate::performance::worker_metrics();
            report(&dir, &current)
        };
        let transfer = (|| -> Result<(), ipc::Failure> {
            publish("checking", ipc::Receipt::default()).map_err(|_| {
                ipc::Failure::new(ipc::FailureStage::Status, ipc::FailureReason::ReportInvalid)
            })?;
            let lease = Lease::open(&auth.target)?;
            let pipe = native::pipe(ipc::PIPE)?;
            publish("connecting", ipc::Receipt::default()).map_err(|_| {
                ipc::Failure::new(ipc::FailureStage::Status, ipc::FailureReason::ReportInvalid)
            })?;
            native::connect(&pipe)?;
            native::client(&pipe, &lease)?;
            // Full uniqueness plus actual client validation once, immediately before writing.
            send_frame(
                &pipe,
                || {
                    lease.validate_for_send()?;
                    native::client(&pipe, &lease)
                },
                &mut publish,
            )
        })();
        if let Err(error) = &transfer {
            current.receipt.fail(error.clone());
            current.phase = if error.reason == ipc::FailureReason::Timeout {
                "timeout"
            } else {
                "failed"
            }
            .into();
            current.elapsed_ms = origin.elapsed().as_millis() as u64;
            current.performance = crate::performance::worker_metrics();
            let _ = report(&dir, &current);
        }
        transfer.map_err(String::from)
    })();
    if result.is_ok() {
        0
    } else {
        64
    }
}

/// A bounded persistent transport used only by the repeating IPC route. The
/// parent sends one length-prefixed authorization record followed by one byte
/// command per frame. The game pipe is connected and client-checked once, then
/// every command is still guarded immediately before its write.
fn persistent_entry() -> i32 {
    let result = (|| -> Result<(), String> {
        let run = std::env::var("CLICKMASTER_IPC_RUN").map_err(|_| "授权缺失。")?;
        std::env::remove_var("CLICKMASTER_IPC_RUN");
        std::env::remove_var("CLICKMASTER_IPC_PERSISTENT");
        if !valid_run(&run) {
            return Err("授权格式无效。".into());
        }
        let exe = std::env::current_exe().map_err(|_| "程序位置无效。")?;
        let dir = crate::data_root(&exe)?.join("ipc").join(run);
        if !dir.join("authorization.json").is_file() {
            return Err("授权缺失。".into());
        }
        let mut input = std::io::stdin().lock();
        let mut length = [0u8; 4];
        input.read_exact(&mut length).map_err(|_| "授权无效。")?;
        let length = u32::from_le_bytes(length);
        if length == 0 || length > 4096 {
            return Err("授权无效。".into());
        }
        let mut bytes = vec![0u8; length as usize];
        input.read_exact(&mut bytes).map_err(|_| "授权无效。")?;
        let auth: Authorization = serde_json::from_slice(&bytes).map_err(|_| "授权无效。")?;
        crate::performance::set_worker(auth.diagnostics != 0);
        let parent = authorize(&dir, &auth)?;
        let origin = Instant::now();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(50));
            if !native::running(&parent) {
                std::process::exit(70);
            }
        });

        let lease = Lease::open(&auth.target)?;
        let pipe = native::pipe(ipc::PIPE)?;
        let mut current = Report::new("checking");
        current.performance_epoch = auth.diagnostics;
        let mut stage = Instant::now();
        let sequence = std::cell::Cell::new(0u64);
        let latest = std::cell::RefCell::new(ipc::Receipt::default());
        let mut publish = |phase: &str, receipt: ipc::Receipt| -> Result<(), String> {
            if current.phase != phase {
                stage = Instant::now();
            }
            current.phase = phase.into();
            current.elapsed_ms = origin.elapsed().as_millis() as u64;
            current.stage_ms = stage.elapsed().as_millis() as u64;
            current.receipt = receipt;
            *latest.borrow_mut() = current.receipt.clone();
            current.sequence = sequence.get();
            current.performance = crate::performance::worker_metrics();
            report(&dir, &current)
        };
        publish("connecting", ipc::Receipt::default())?;
        native::connect(&pipe)?;
        native::client(&pipe, &lease)?;
        let connected = ipc::Receipt {
            connected: true,
            ..Default::default()
        };
        publish("connected", connected.clone())?;
        loop {
            let mut command = [0u8; 1];
            if input.read_exact(&mut command).is_err() {
                break;
            }
            if command[0] != 1 {
                return Err("命令无效。".into());
            }
            sequence.set(sequence.get().checked_add(1).ok_or("发送计数达到上限。")?);
            let transfer = send_persistent_frame(
                &pipe,
                || {
                    lease.validate_for_send()?;
                    native::client(&pipe, &lease)
                },
                &mut publish,
            );
            if let Err(error) = transfer {
                let mut failed = latest.borrow().clone();
                failed.fail(error.clone());
                let _ = publish("failed", failed);
                return Err(error.into());
            }
        }
        Ok(())
    })();
    if result.is_ok() {
        0
    } else {
        64
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_names_cannot_escape_executable_data_root() {
        assert!(valid_run(&"A".repeat(64)));
        for name in ["../escape", "", &"a".repeat(64), &"G".repeat(64)] {
            assert!(!valid_run(name));
        }
    }
    #[test]
    fn secret_and_target_are_not_in_manifest_or_report() {
        let text = serde_json::to_string(&Manifest {
            token_hash: "HASH".into(),
            binary_hash: "HASH".into(),
        })
        .unwrap();
        assert!(!text.contains("parent"));
        assert!(!text.contains("target"));
        assert!(!serde_json::to_string(&Report::new("connecting"))
            .unwrap()
            .contains("pid"));
    }
    #[test]
    fn denied_final_guard_prevents_native_write_and_preserves_fixed_reason() {
        let name = format!(
            r"\\.\pipe\ClickMaster-OfflineFixture-{}",
            native::random().unwrap()
        );
        let pipe = native::pipe(&name).unwrap();
        let failure = ipc::Failure::new(
            ipc::FailureStage::Client,
            ipc::FailureReason::ClientMismatch,
        );
        let mut phases = Vec::new();
        let result = send_frame(
            &pipe,
            || Err(failure.clone()),
            |phase, receipt| {
                phases.push((phase.to_string(), receipt));
                Ok(())
            },
        );
        assert_eq!(result.unwrap_err(), failure);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].0, "writing");
        assert!(!phases[0].1.written);
    }
}
