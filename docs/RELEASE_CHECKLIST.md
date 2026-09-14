# v0.6.2 打 tag 前验收清单 / Pre-tag checklist

打 `v0.6.2` 并推送到 GitHub 之前勾完本页。不要移动 `v0.5.0`、`v0.5.1`、`v0.6.0`、`v0.6.1`。流程见 [RELEASE.md](RELEASE.md)。
Complete this page before tagging `v0.6.2`. Do not move `v0.5.0`, `v0.5.1`, `v0.6.0`, or `v0.6.1`. See [RELEASE.md](RELEASE.md) for the publish flow.

## 文档 / Docs

- [x] `README.md` 与 `README_EN.md` 特性、技术栈、架构、目录结构一致
- [x] `Cargo.toml` 版本为 `0.6.2`，与 README / RELEASE / Inno / Info.plist 一致
- [ ] 若 chrome 有可见变化，重拍 `docs/screenshots/main_zh.png`、`main_en.png`
- [ ] 截图分别用中文、英文界面打开对应 README，大纲标题为「特性 / Features」，无叠字

## 本地检查 / Local CI parity

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --locked
```

- [ ] 以上四条全部通过

## CodeMirror 6 门禁 / Editor kernel gates

- [ ] vendor IIFE 暴露 `window.MarkdownMonkeyCM`
- [ ] `_mm_*` 桥仍由 `editor_codemirror.js` 提供，含 `_mm_clipboardAction`
- [ ] 内核挂上后底层 textarea 不可见，行号与 Markdown 着色正常
- [ ] 深色 / 浅色切换后正文仍可见
- [ ] Ctrl/⌘+Z / Y 走内核历史；切标签后历史不串文件
- [ ] `](` 图片路径补全、悬停预览、缺失路径轻量 lint 可用
- [ ] 编辑器右键：复制 / 剪切 / 粘贴；无选区时 AI 助手为灰；有选区可跑预设
- [ ] 设置里的自动换行、行号会作用到内核

## 实机界面 / Desktop smoke

用 `cargo run` 或 release 二进制，**从磁盘重新打开** `README.md` 与 `README_EN.md`。

- [ ] 编辑器、预览、大纲标题为「特性」/ `Features`
- [ ] 相对路径图片在预览中能显示
- [ ] `$E=mc^2$` 与 Mermaid 预览正常
- [ ] 工具栏中英文切换后面板文案正确

## 文件与会话 / Files and session

- [ ] 打开 / 保存 / 另存为；未保存关闭有确认
- [ ] 拖放 `.md` / `.txt` 会开标签；粘贴图片会写入工作区并插入 Markdown
- [ ] 重启后恢复工作区、标签和未保存草稿（设置里可关）

## 搜索与 AI / Search and AI

- [ ] Ctrl/⌘+F 文档内搜索；Ctrl/⌘+Shift+F 工作区搜索
- [ ] Ctrl/⌘+J / 工具栏打开的是聊天：回复在聊天窗流式出现，记录显示用户原话，不出现「请将以下文本翻译成…」这类任务提示
- [ ] 聊天附带选区或光标附近上下文，设置里的字数上限生效
- [ ] 右键「译成英文」结果是译文并走结果弹窗，不混进聊天记录
- [ ] 未配置 Key 时提示明确（不把 Key 写进 `settings.json`）

## 导出与安装包 / Export and packages

- [ ] 导出 HTML / 纯文本 / 打印框可用
- [ ] 不打 tag，先用 Release 工作流 `workflow_dispatch` 打一版产物并本地试装：
  - [ ] Windows：zip 可解压运行，`*-setup.exe` 能装能开
  - [ ] Linux：`.tar.gz` 与 `.deb` 能装能开（有桌面项）
  - [ ] macOS：`.app` 能从 tar 解开后启动

## 打 tag / Tag

全部勾选后再执行：

```powershell
git tag v0.6.2
git push origin v0.6.2
```

- [ ] GitHub Release 已生成，三个平台附件和 checksum 齐全
- [ ] 本清单无需随 tag 清空；下次版本复制后改标题即可
