//! 用户目录解析（不依赖 dirs crate）
//! Resolve user directories without the `dirs` crate

use std::path::PathBuf;

/// 返回平台配置根目录（不含应用名）
/// Return the platform config root (without the app folder name)
pub fn config_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg));
        }
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
    }
    #[cfg(not(any(windows, unix)))]
    {
        None
    }
}

/// 返回图片目录，找不到则 `None`
/// Return the pictures directory, or `None` if it cannot be resolved
pub fn pictures_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join("Pictures"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Pictures"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(xdg) = std::env::var_os("XDG_PICTURES_DIR") {
            return Some(PathBuf::from(xdg));
        }
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Pictures"))
    }
    #[cfg(not(any(windows, unix)))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_is_some_on_supported_platforms() {
        assert!(config_dir().is_some());
    }
}
