# v0.6.6 打 tag 前验收清单 / Pre-tag checklist

打 `v0.6.6` 并推送到 GitHub 之前勾完本页。不要移动 `v0.5.0`、`v0.5.1`、`v0.6.0`、`v0.6.1`、`v0.6.2`、`v0.6.3`、`v0.6.4`、`v0.6.5`。流程见 [RELEASE.md](RELEASE.md)。
Complete this page before tagging `v0.6.6`. Do not move `v0.5.0`, `v0.5.1`, `v0.6.0`, `v0.6.1`, `v0.6.2`, `v0.6.3`, `v0.6.4`, or `v0.6.5`. See [RELEASE.md](RELEASE.md) for the publish flow.

本版让续写、优化、修正语法共用语气菜单，并把上次选择记在每个文档和会话里。结果窗在不能对照全文时标出来源摘要。文件树标出未保存和外部已改的文件。拼写检查做成设置，默认关闭。点选语气后结果窗仍会打开。
This release shares one tone menu across Continue, Improve, and Fix Grammar, and remembers the last choice on each document and in the session. The result modal shows a source excerpt when a full compare is unavailable. The file tree marks unsaved tabs and files changed on disk. Spell check is a setting and defaults to off. Choosing a tone still opens the result modal.

## 文档 / Docs

- [x] `Cargo.toml` 版本为 `0.6.6`，与 README / RELEASE / Inno / Info.plist 一致
- [x] 语气菜单、来源摘要、文件树标记、拼写检查默认关闭已写入 README；主界面截图仍沿用上一版

## 本地检查 / Local CI parity

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo audit
cargo build --release --locked
```

- [x] `cargo fmt`、`cargo clippy -D warnings`、`cargo test --all-targets` 已通过（320 tests）
- [x] `cargo audit` 退出码为 0（仅有既有 warning），`cargo build --release --locked` 已通过

## 打 tag / Tag

```powershell
git tag v0.6.6
git push origin v0.6.6
```

- [ ] GitHub Release 已生成，三个平台附件和 checksum 齐全
- [ ] 本清单无需随 tag 清空；下次版本复制后改标题即可
