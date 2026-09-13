# v0.5.1 打 tag 前验收清单 / Pre-tag checklist

打 `v0.5.1` 并推送到 GitHub 之前勾完本页。流程见 [RELEASE.md](RELEASE.md)。
Complete this page before tagging `v0.5.1`. See [RELEASE.md](RELEASE.md) for the publish flow.

## 文档 / Docs

- [x] `README.md` 与 `README_EN.md` 特性、技术栈、架构、目录结构一致
- [x] `Cargo.toml` 版本为 `0.5.1`，与 README / RELEASE / Inno / Info.plist 一致
- [x] `docs/screenshots/main_zh.png`、`main_en.png` 已用当前 UI **重拍**
- [x] 截图分别用中文、英文界面打开对应 README，大纲标题为「特性 / Features」，无叠字

## 本地检查 / Local CI parity

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --locked
```

- [x] 以上四条全部通过

## 实机界面 / Desktop smoke

用 `cargo run` 或 release 二进制，**从磁盘重新打开** `README.md` 与 `README_EN.md`（不要保存仍乱码的缓冲区）。

- [x] 编辑器、预览、大纲标题为「特性」/ `Features`，无叠字、无双行号
- [x] CodeMirror 行号与 Markdown 着色正常，底层 textarea 不可见
- [x] 预览表格列宽可读，中西文混排不挤成竖条
- [x] 相对路径图片（如 `docs/screenshots/main_*.png`）在预览中能显示
- [x] 任务列表、脚注、删除线渲染正确
- [x] `$E=mc^2$` 与 `$$` 块级公式能出预览（使用打包的 KaTeX woff2）
- [x] ` ```mermaid ` 流程图能出图，失败时仍显示源码
- [x] 预览 rust/js 代码块有 syntect 高亮
- [x] 大纲点击跳到对应标题；同步滚动开关有效
- [x] 深色 / 浅色 / 跟随系统切换后编辑器主题跟着变
- [x] 工具栏中英文切换后面板文案正确

## 文件与会话 / Files and session

- [x] 打开 / 保存 / 另存为；未保存关闭有确认
- [x] 拖放 `.md` / `.txt` 会开标签；粘贴图片会写入工作区并插入 Markdown
- [x] 重启后恢复工作区、标签和未保存草稿（设置里可关）
- [x] UTF-8 / GBK 文件能打开且不乱码

## 搜索与 AI / Search and AI

- [x] Ctrl/⌘+F 文档内搜索高亮与跳转
- [x] Ctrl/⌘+Shift+F 工作区搜索能命中已打开标签缓冲
- [x] AI 面板能开能关；未配置 Key 时提示明确（不把 Key 写进 `settings.json`）

## 导出与安装包 / Export and packages

- [x] 导出 HTML 含高亮 / 公式 / Mermaid 脚本；可选打包本地图片到 `{stem}_files/`
- [x] 导出纯文本内容完整
- [x] 导出菜单「打印 / PDF」或 Ctrl+Shift+P 能打开系统打印框
- [x] 不打 tag，先用 Release 工作流 `workflow_dispatch` 打一版产物并本地试装：
  - [x] Windows：zip 可解压运行，`*-setup.exe` 能装能开
  - [x] Linux：`.tar.gz` 与 `.deb` 能装能开（有桌面项）
  - [x] macOS：`.app` 能从 tar 解开后启动

## 打 tag / Tag

全部勾选后再执行：

```powershell
git tag v0.5.1
git push origin v0.5.1
```

- [ ] GitHub Release 已生成，三个平台附件和 checksum 齐全
- [ ] 本清单无需随 tag 清空；下次版本复制后改标题即可
