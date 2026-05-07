#![windows_subsystem = "windows"]

slint::include_modules!();

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use slint::{ModelRc, SharedString, Timer, VecModel};

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn conf_path() -> PathBuf {
    exe_dir().join("cc-swap.conf")
}

fn settings_dir() -> PathBuf {
    exe_dir().join("settings")
}

fn default_claude_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}

fn load_or_pick_target() -> Option<PathBuf> {
    if let Ok(s) = std::fs::read_to_string(conf_path()) {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }
    let picked = rfd::FileDialog::new()
        .set_title("选择 Claude Code 的 settings.json")
        .set_directory(default_claude_dir())
        .set_file_name("settings.json")
        .save_file()?;
    let _ = std::fs::write(conf_path(), picked.to_string_lossy().as_bytes());
    Some(picked)
}

fn scan_settings() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(settings_dir())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    v.sort();
    v
}

fn switch_profile(target: &Path, src: &Path) -> std::io::Result<()> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if target.exists() {
        let bak_name = format!(
            "{}.bak",
            target
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "settings.json".to_string())
        );
        let bak = target.with_file_name(bak_name);
        std::fs::copy(target, &bak)?;
    }
    std::fs::copy(src, target)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = match load_or_pick_target() {
        Some(p) => p,
        None => return Ok(()),
    };
    std::fs::create_dir_all(settings_dir())?;
    let files = scan_settings();
    let names: Vec<SharedString> = files
        .iter()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
                .into()
        })
        .collect();

    let ui = AppWindow::new()?;
    ui.set_target_path(target.to_string_lossy().to_string().into());
    ui.set_profiles(ModelRc::from(Rc::new(VecModel::from(names))));

    let ui_weak = ui.as_weak();
    ui.on_profile_clicked(move |idx| {
        let Some(src) = files.get(idx as usize) else {
            return;
        };
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };
        match switch_profile(&target, src) {
            Ok(_) => {
                let name = src
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                ui.set_status(format!("已切换到 {}", name).into());
                Timer::single_shot(Duration::from_millis(500), || {
                    let _ = slint::quit_event_loop();
                });
            }
            Err(e) => {
                ui.set_status(format!("失败: {}", e).into());
            }
        }
    });

    ui.run()?;
    Ok(())
}
