# v0.6.4 打 tag 前验收清单 / Pre-tag checklist

打 `v0.6.4` 并推送到 GitHub 之前勾完本页。不要移动 `v0.5.0`、`v0.5.1`、`v0.6.0`、`v0.6.1`、`v0.6.2`、`v0.6.3`。流程见 [RELEASE.md](RELEASE.md)。
Complete this page before tagging `v0.6.4`. Do not move `v0.5.0`, `v0.5.1`, `v0.6.0`, `v0.6.1`, `v0.6.2`, or `v0.6.3`. See [RELEASE.md](RELEASE.md) for the publish flow.

本版是安全补丁：`rustls` 0.23.43 → 0.23.45（RUSTSEC-2026-0285）。界面无变化，沿用 0.6.3 截图。
This release is a security patch: `rustls` 0.23.43 → 0.23.45 (RUSTSEC-2026-0285). UI is unchanged; keep the 0.6.3 screenshots.

## 文档 / Docs

- [x] `README.md` 与 `README_EN.md` 特性、技术栈、架构、目录结构一致
- [x] `Cargo.toml` 版本为 `0.6.4`，与 README / RELEASE / Inno / Info.plist 一致
- [x] 界面无可见变化，不重拍截图
- [x] `Cargo.lock` 中 `rustls` 为 `0.23.45`；`cargo audit` 无 vulnerability（允许的 warning 除外）

## 本地检查 / Local CI parity

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo audit
```

- [x] 以上检查在 rustls 升级后已通过（307 tests；audit 仅剩 allowed warnings）

## 打 tag / Tag

```powershell
git tag v0.6.4
git push origin v0.6.4
```

- [ ] GitHub Release 已生成，三个平台附件和 checksum 齐全
- [ ] 本清单无需随 tag 清空；下次版本复制后改标题即可
