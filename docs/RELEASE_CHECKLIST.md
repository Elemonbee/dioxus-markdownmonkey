# v0.6.5 打 tag 前验收清单 / Pre-tag checklist

打 `v0.6.5` 并推送到 GitHub 之前勾完本页。不要移动 `v0.5.0`、`v0.5.1`、`v0.6.0`、`v0.6.1`、`v0.6.2`、`v0.6.3`、`v0.6.4`。流程见 [RELEASE.md](RELEASE.md)。
Complete this page before tagging `v0.6.5`. Do not move `v0.5.0`, `v0.5.1`, `v0.6.0`, `v0.6.1`, `v0.6.2`, `v0.6.3`, or `v0.6.4`. See [RELEASE.md](RELEASE.md) for the publish flow.

本版让续写、翻译等预设结果重新显示，并为续写增加风格子菜单。同时覆盖多标签的监视与自动保存、驱逐标签时保留撤销历史、稳定的 AI 会话键、HTML 导出使用本地公式与图表、预览放行安全 HTML，以及编辑器行号与 CodeMirror 对齐。
This release shows continue and translate output again and adds a continue-writing style menu. It also covers multi-tab watch and autosave, undo history kept when a tab is evicted, stable AI session keys, local math and diagrams in HTML export, a safe HTML preview subset, and editor line numbers aligned with CodeMirror.

## 文档 / Docs

- [x] `Cargo.toml` 版本为 `0.6.5`，与 README / RELEASE / Inno / Info.plist 一致
- [x] 续写菜单有可见变化；主界面截图仍沿用上一版

## 本地检查 / Local CI parity

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo audit
```

- [x] `cargo fmt`、`cargo clippy -D warnings`、`cargo test --all-targets` 已通过（313 tests）
- [ ] `cargo audit` 与 `cargo build --release --locked` 未在本次打 tag 前重跑

## 打 tag / Tag

```powershell
git tag v0.6.5
git push origin v0.6.5
```

- [ ] GitHub Release 已生成，三个平台附件和 checksum 齐全
- [ ] 本清单无需随 tag 清空；下次版本复制后改标题即可
