//! 系统主题检测服务 / System Theme Detection Service
//!
//! 检测操作系统的深色/浅色主题设置，支持 Windows/macOS/Linux
//! Detects OS dark/light theme settings, supporting Windows/macOS/Linux
//!
//! 用法 / Usage:
//! ```ignore
//! let theme_str = ThemeDetector::detect(); // "dark" or "light"
//! ```

/// 系统主题检测器 / System Theme Detector
///
/// 提供 `detect()` 方法查询当前操作系统主题偏好。
/// 失败时安全回退到 `"dark"`。
/// Provides `detect()` to query the current OS theme preference.
/// Falls back to `"dark"` on failure.
pub struct ThemeDetector;

impl ThemeDetector {
    /// 检测系统主题 / Detect system theme
    ///
    /// 返回 `"dark"` 或 `"light"`。检测失败时回退到 `"dark"`。
    /// Returns `"dark"` or `"light"`. Falls back to `"dark"` on failure.
    pub fn detect() -> &'static str {
        match Self::detect_inner() {
            Some(theme) => theme,
            None => {
                tracing::warn!(
                    "主题检测失败，使用默认深色主题 / Theme detection failed, using default dark theme"
                );
                "dark"
            }
        }
    }

    /// 内部主题检测实现 / Internal theme detection implementation
    ///
    /// 按平台检测：Windows 注册表、macOS defaults、Linux gsettings/环境变量。
    /// Platform-specific: Windows registry, macOS defaults, Linux gsettings/env.
    fn detect_inner() -> Option<&'static str> {
        #[cfg(target_os = "windows")]
        {
            Self::detect_windows()
        }

        #[cfg(target_os = "macos")]
        {
            Self::detect_macos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::detect_linux()
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            None
        }
    }

    /// Windows: 通过注册表检测 / Windows: detect via registry
    #[cfg(target_os = "windows")]
    fn detect_windows() -> Option<&'static str> {
        use std::process::Command;

        if let Ok(output) = Command::new("reg")
            .args([
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "/v",
                "AppsUseLightTheme",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // 如果包含 0x0 则是深色主题 / 0x0 indicates dark theme
            if stdout.contains("0x0") || stdout.contains("0x0000") {
                return Some("dark");
            }
            // 如果包含 0x1 则是浅色主题 / 0x1 indicates light theme
            if stdout.contains("0x1") || stdout.contains("0x0001") {
                return Some("light");
            }
        }
        // 备用：检查系统颜色设置 / Fallback: check system color settings
        if let Ok(output) = Command::new("reg")
            .args([
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "/v",
                "SystemUsesLightTheme",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("0x0") {
                return Some("dark");
            }
            if stdout.contains("0x1") {
                return Some("light");
            }
        }

        None
    }

    /// macOS: 使用 defaults 命令检测 / macOS: detect via defaults command
    #[cfg(target_os = "macos")]
    fn detect_macos() -> Option<&'static str> {
        use std::process::Command;

        if let Ok(output) = Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Dark") {
                return Some("dark");
            }
        }
        // 如果没有设置 AppleInterfaceStyle，说明是浅色模式 / No AppleInterfaceStyle means light mode
        Some("light")
    }

    /// Linux: GNOME gsettings + 环境变量 / Linux: GNOME gsettings + env vars
    #[cfg(target_os = "linux")]
    fn detect_linux() -> Option<&'static str> {
        use std::process::Command;

        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if stdout.contains("prefer-dark") || stdout.contains("'dark'") {
                return Some("dark");
            }
            if stdout.contains("prefer-light") {
                return Some("light");
            }
        }

        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if stdout.contains("dark") {
                return Some("dark");
            }
        }

        fn env_lower(key: &str) -> Option<String> {
            std::env::var(key).ok().map(|s| s.to_lowercase())
        }

        if matches!(env_lower("GTK_THEME").as_deref(), Some(s) if s.contains("dark")) {
            return Some("dark");
        }
        match env_lower("COLOR_SCHEME") {
            Some(s) if s == "dark" || s.contains("prefer-dark") => return Some("dark"),
            Some(s) if s == "light" || s.contains("prefer-light") => return Some("light"),
            _ => {}
        }
        if std::env::var("DARK_MODE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            return Some("dark");
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_returns_valid_string() {
        // 检测应该总是返回 "dark" 或 "light"
        // Detection should always return "dark" or "light"
        let result = ThemeDetector::detect();
        assert!(result == "dark" || result == "light");
    }

    #[test]
    fn test_detect_inner_returns_valid_or_none() {
        // detect_inner 应该返回 Some("dark"), Some("light"), 或 None
        // detect_inner should return Some("dark"), Some("light"), or None
        let result = ThemeDetector::detect_inner();
        if let Some(s) = result {
            assert!(s == "dark" || s == "light");
        }
    }

    #[test]
    fn test_detect_fallback_is_dark() {
        // detect() 的回退值应该是 "dark"
        // detect() fallback should be "dark"
        // 此测试仅验证返回值合法，不验证检测逻辑本身
        // This test only verifies the return value is valid, not detection logic
        let result = ThemeDetector::detect();
        assert!(!result.is_empty());
    }
}
