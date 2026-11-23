use std::path::Path;
use std::process::{Command, Stdio};

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::models::{ActionResult, SystemThemeInfo};

pub fn stop_process(pid: u32) -> ActionResult {
    send_termination_signal(pid, false)
}

pub fn kill_process(pid: u32) -> ActionResult {
    send_termination_signal(pid, true)
}

fn send_termination_signal(pid: u32, force: bool) -> ActionResult {
    if pid == 0 {
        return ActionResult {
            ok: false,
            message: "Refusing to signal PID 0.".to_string(),
            requires_elevation: false,
        };
    }
    if pid == std::process::id() {
        return ActionResult {
            ok: false,
            message: "Refusing to terminate iPort itself.".to_string(),
            requires_elevation: false,
        };
    }

    #[cfg(unix)]
    {
        let sig = if force { libc::SIGKILL } else { libc::SIGTERM };
        let sig_name = if force { "SIGKILL" } else { "SIGTERM" };
        let rc = unsafe { libc::kill(pid as libc::pid_t, sig) };
        if rc == 0 {
            return ActionResult {
                ok: true,
                message: format!("Sent {sig_name} to PID {pid}."),
                requires_elevation: false,
            };
        }

        let err = std::io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(libc::EPERM) => ActionResult {
                ok: false,
                message: format!(
                    "Permission denied signaling PID {pid}. Elevated privileges are required."
                ),
                requires_elevation: true,
            },
            Some(libc::ESRCH) => ActionResult {
                ok: false,
                message: format!("Process PID {pid} has already exited."),
                requires_elevation: false,
            },
            _ => ActionResult {
                ok: false,
                message: format!("Failed to send {sig_name} to PID {pid}: {err}"),
                requires_elevation: false,
            },
        };
    }

    #[cfg(windows)]
    {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/PID", &pid.to_string()]);
        if force {
            cmd.arg("/F");
        }
        match cmd.output() {
            Ok(output) if output.status.success() => ActionResult {
                ok: true,
                message: format!(
                    "{} PID {pid}.",
                    if force { "Force killed" } else { "Stopped" }
                ),
                requires_elevation: false,
            },
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let needs_admin = stderr.to_ascii_lowercase().contains("access is denied");
                ActionResult {
                    ok: false,
                    message: if stderr.is_empty() {
                        format!("taskkill failed for PID {pid}.")
                    } else {
                        stderr
                    },
                    requires_elevation: needs_admin,
                }
            }
            Err(err) => ActionResult {
                ok: false,
                message: format!("Failed to run taskkill for PID {pid}: {err}"),
                requires_elevation: false,
            },
        }
    }

    #[cfg(all(not(unix), not(windows)))]
    {
        let _ = force;
        ActionResult {
            ok: false,
            message: "Process signaling is unsupported on this OS.".to_string(),
            requires_elevation: false,
        }
    }
}

pub fn build_browser_url(port: u16, bind_address: &str, use_https: Option<bool>) -> String {
    let is_https = use_https.unwrap_or(port == 443 || port == 8443);
    let scheme = if is_https { "https" } else { "http" };
    let trimmed = bind_address.trim();

    let host = if trimmed.is_empty() || matches!(trimmed, "0.0.0.0" | "::" | "*" | "[::]") {
        "localhost".to_string()
    } else if trimmed.contains(':') && !trimmed.starts_with('[') {
        format!("[{trimmed}]")
    } else {
        trimmed.to_string()
    };

    format!("{scheme}://{host}:{port}")
}

pub fn open_in_browser(
    app: &AppHandle,
    port: u16,
    bind_address: &str,
    use_https: Option<bool>,
) -> ActionResult {
    let url = build_browser_url(port, bind_address, use_https);
    match app.opener().open_url(url.clone(), None::<&str>) {
        Ok(()) => ActionResult {
            ok: true,
            message: format!("Opened {url}"),
            requires_elevation: false,
        },
        Err(err) => ActionResult {
            ok: false,
            message: format!("Could not open {url}: {err}"),
            requires_elevation: false,
        },
    }
}

pub fn reveal_in_folder(app: &AppHandle, path: &str) -> ActionResult {
    let target = Path::new(path);
    if !target.exists() {
        return ActionResult {
            ok: false,
            message: format!("Directory does not exist: {path}"),
            requires_elevation: false,
        };
    }

    match app.opener().reveal_item_in_dir(target) {
        Ok(()) => ActionResult {
            ok: true,
            message: format!("Opened {path}"),
            requires_elevation: false,
        },
        Err(err) => ActionResult {
            ok: false,
            message: format!("Could not open folder {path}: {err}"),
            requires_elevation: false,
        },
    }
}

pub fn open_in_ide(path: &str, ide: &str) -> ActionResult {
    let target = Path::new(path);
    if !target.exists() {
        return ActionResult {
            ok: false,
            message: format!("Working directory does not exist: {path}"),
            requires_elevation: false,
        };
    }

    let bin = match ide.trim() {
        "code" => "code",
        "cursor" => "cursor",
        "zed" => "zed",
        "webstorm" => "webstorm",
        "idea" => "idea",
        other => {
            return ActionResult {
                ok: false,
                message: format!("Unsupported IDE selection: {other}"),
                requires_elevation: false,
            };
        }
    };

    match Command::new(bin)
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_) => ActionResult {
            ok: true,
            message: format!("Opened project in {bin}."),
            requires_elevation: false,
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => ActionResult {
            ok: false,
            message: format!("'{bin}' CLI was not found in PATH."),
            requires_elevation: false,
        },
        Err(err) => ActionResult {
            ok: false,
            message: format!("Failed to launch {bin}: {err}"),
            requires_elevation: false,
        },
    }
}

pub fn read_system_theme() -> SystemThemeInfo {
    #[cfg(target_os = "linux")]
    {
        let gtk_theme = read_gsettings("gtk-theme").unwrap_or_else(|| "Adwaita".to_string());
        let color_scheme = read_gsettings("color-scheme").unwrap_or_else(|| "default".to_string());
        let gnome_accent = read_gsettings("accent-color");

        let (accent_hex, accent_dark_hex) = if let Some(accent) = gnome_accent.as_deref() {
            resolve_gnome_accent_key(accent).unwrap_or_else(|| resolve_gnome_accent(&gtk_theme))
        } else {
            resolve_gnome_accent(&gtk_theme)
        };

        SystemThemeInfo {
            color_scheme,
            gtk_theme,
            accent_hex: accent_hex.to_string(),
            accent_dark_hex: accent_dark_hex.to_string(),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        SystemThemeInfo {
            color_scheme: "default".to_string(),
            gtk_theme: "Adwaita".to_string(),
            accent_hex: "#3584E4".to_string(),
            accent_dark_hex: "#78AEED".to_string(),
        }
    }
}

#[cfg(target_os = "linux")]
fn read_gsettings(key: &str) -> Option<String> {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", key])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    let cleaned = raw.trim().trim_matches('\'').trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

#[cfg(target_os = "linux")]
fn resolve_gnome_accent_key(key: &str) -> Option<(&'static str, &'static str)> {
    match key.trim().to_ascii_lowercase().as_str() {
        "blue" => Some(("#3584E4", "#78AEED")),
        "teal" => Some(("#2190A4", "#56C2D6")),
        "green" => Some(("#26A269", "#57E389")),
        "yellow" => Some(("#CD9309", "#F8E45C")),
        "orange" => Some(("#E66100", "#FFA348")),
        "red" => Some(("#E01B24", "#FF7B63")),
        "pink" => Some(("#D56199", "#F38FB9")),
        "purple" => Some(("#9141AC", "#DC8ADD")),
        "slate" => Some(("#6F8396", "#9AAAB9")),
        _ => None,
    }
}

pub fn resolve_gnome_accent(gtk_theme: &str) -> (&'static str, &'static str) {
    let lower = gtk_theme.trim().to_ascii_lowercase();
    let base = lower
        .strip_suffix("-dark")
        .or_else(|| lower.strip_suffix("-light"))
        .unwrap_or(lower.as_str());

    match base {
        "yaru" => ("#E95420", "#F27447"),
        "yaru-blue" => ("#0073E5", "#5BA4F5"),
        "yaru-bark" => ("#787859", "#A5A585"),
        "yaru-sage" => ("#657B69", "#8FA893"),
        "yaru-olive" => ("#4B8501", "#78B82A"),
        "yaru-viridian" => ("#03875B", "#34B88A"),
        "yaru-prussiangreen" => ("#308280", "#5CB0AE"),
        "yaru-purple" => ("#7764D8", "#9D8DF0"),
        "yaru-magenta" => ("#B34CB3", "#D47AD4"),
        "yaru-red" => ("#DA3450", "#F2667D"),
        _ => ("#3584E4", "#78AEED"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_ubuntu_yaru_and_adwaita_accents() {
        assert_eq!(resolve_gnome_accent("Yaru"), ("#E95420", "#F27447"));
        assert_eq!(resolve_gnome_accent("Yaru-dark"), ("#E95420", "#F27447"));
        assert_eq!(resolve_gnome_accent("Yaru-blue-dark"), ("#0073E5", "#5BA4F5"));
        assert_eq!(resolve_gnome_accent("Yaru-sage"), ("#657B69", "#8FA893"));
        assert_eq!(resolve_gnome_accent("Adwaita"), ("#3584E4", "#78AEED"));
    }

    #[test]
    fn builds_browser_urls_cleanly() {
        assert_eq!(build_browser_url(3000, "0.0.0.0", None), "http://localhost:3000");
        assert_eq!(build_browser_url(8000, "::", None), "http://localhost:8000");
        assert_eq!(build_browser_url(5173, "127.0.0.1", None), "http://127.0.0.1:5173");
        assert_eq!(build_browser_url(8443, "::1", None), "https://[::1]:8443");
        assert_eq!(build_browser_url(3000, "127.0.0.1", Some(true)), "https://127.0.0.1:3000");
    }
}
