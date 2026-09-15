//! Windows 资源编译：把应用图标嵌入 exe
//! Windows resource build: embed the app icon into the exe

fn main() {
    println!("cargo:rerun-if-changed=packaging/windows/icon.ico");
    embed_windows_icon();
}

/// 仅在 Windows 上把 `icon.ico` 编进可执行文件
/// Embed `icon.ico` into the executable on Windows only
fn embed_windows_icon() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("packaging/windows/icon.ico");
        res.compile()
            .expect("failed to embed Windows application icon");
    }
}
