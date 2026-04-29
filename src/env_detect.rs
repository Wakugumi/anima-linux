use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayEnv {
    /// Running on a native X11 session. All GTK window hints work.
    X11,
    /// Running via XWayland with GDK_BACKEND=x11 explicitly set by user/launcher.
    /// All GTK window hints work.
    XWaylandExplicit,
    /// Running via XWayland but GDK_BACKEND was not explicitly set.
    /// GTK window hints still work, but we recommend the user set GDK_BACKEND=x11.
    XWaylandImplicit,
    /// Running on native Wayland. GTK3 window hints (skip_taskbar, keep_above) are ignored.
    NativeWayland,
    /// Could not determine the environment. Hints are applied but may not work.
    Unknown,
}

#[allow(dead_code)]
impl DisplayEnv {

    pub fn taskbar_hiding_possible(&self) -> bool {
        matches!(self, DisplayEnv::X11 | DisplayEnv::XWaylandExplicit | DisplayEnv::XWaylandImplicit)
    }

    pub fn is_x11_or_xwayland(&self) -> bool {
        matches!(self, DisplayEnv::X11 | DisplayEnv::XWaylandExplicit | DisplayEnv::XWaylandImplicit)
    }

    pub fn label(&self) -> &'static str {
        match self {
            DisplayEnv::X11 => "X11 (native)",
            DisplayEnv::XWaylandExplicit => "XWayland (configured)",
            DisplayEnv::XWaylandImplicit => "XWayland (auto-detected)",
            DisplayEnv::NativeWayland => "Native Wayland",
            DisplayEnv::Unknown => "Unknown",
        }
    }
}

/// Detect the current display environment by inspecting environment variables.
pub fn detect() -> DisplayEnv {
    let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let has_wayland = env::var("WAYLAND_DISPLAY").is_ok();
    let has_display = env::var("DISPLAY").is_ok();
    let gdk_backend = env::var("GDK_BACKEND").unwrap_or_default();

    if session_type == "wayland" || has_wayland {
        if has_display {
            // Wayland session but DISPLAY is set,  running under XWayland
            if gdk_backend == "x11" {
                DisplayEnv::XWaylandExplicit
            } else {
                DisplayEnv::XWaylandImplicit
            }
        } else {
            // Pure Wayland, no XWayland bridge
            DisplayEnv::NativeWayland
        }
    } else if has_display || session_type == "x11" {
        DisplayEnv::X11
    } else {
        DisplayEnv::Unknown
    }
}

pub fn is_gnome() -> bool {
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
    let gnome_session = env::var("GNOME_DESKTOP_SESSION_ID").is_ok();
    desktop.contains("gnome") || gnome_session
}

/// Read the current GNOME always-on-top keybinding from gsettings.
/// Returns None if gsettings is unavailable or no binding is set.
pub fn read_gnome_always_on_top_key() -> Option<String> {
    let output = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.wm.keybindings", "always-on-top"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // gsettings returns e.g. ['<Control><Super>t'] or @as []
    if raw.contains('@') || raw == "[]" {
        None
    } else {
        // Strip outer ['...']
        let inner = raw.trim_start_matches('[').trim_end_matches(']');
        let key = inner.trim().trim_matches('\'').to_string();
        if key.is_empty() { None } else { Some(key) }
    }
}

/// Set the GNOME always-on-top keybinding via gsettings.
/// Returns true on success.
pub fn set_gnome_always_on_top_key(key: &str) -> bool {
    // Sanitize: key should only contain alphanumeric, <, >, -, _
    let safe = key.chars().all(|c| c.is_alphanumeric() || "<>-_".contains(c));
    if !safe || key.len() > 64 {
        eprintln!("Rejected unsafe gsettings key input: {:?}", key);
        return false;
    }

    let value = format!("['{}']", key);
    std::process::Command::new("gsettings")
        .args(["set", "org.gnome.desktop.wm.keybindings", "always-on-top", &value])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
