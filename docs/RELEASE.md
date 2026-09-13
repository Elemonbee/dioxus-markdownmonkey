# Release Guide

MarkdownMonkey publishes desktop release packages through GitHub Actions for
**Windows**, **Linux**, and **macOS**.

Current app version: **0.5.0** (see `Cargo.toml`). CI uses the `stable` Rust
toolchain; building from source needs **Rust 1.88+**.

Before creating the `v0.5.0` tag, complete
**[RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md)** (docs, desktop smoke, packages).

## Local Verification

Run the same checks used by CI before cutting a release:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --locked
```

Release binaries are generated at:

```text
target/release/markdownmonkey.exe   # Windows
target/release/markdownmonkey       # Linux / macOS
```

## Manual Artifact Build

Use the `Release` workflow's `workflow_dispatch` trigger from GitHub Actions.
Optionally provide a version label such as `v0.5.0`. This creates workflow
artifacts containing portable archives **and** native installers:

- Windows: `MarkdownMonkey-<version>-windows-x64.zip` + `...-setup.exe`
- Linux: `MarkdownMonkey-<version>-linux-x64.tar.gz` + `.deb`
- macOS: `MarkdownMonkey-<version>-macos-arm64.tar.gz` (or `macos-x64`) containing `MarkdownMonkey.app`

Manual workflow runs do not create a GitHub Release.

## Tagged Release

Create and push a version tag:

```powershell
git tag v0.5.0
git push origin v0.5.0
```

Tags matching `v*` build all platform packages, upload workflow artifacts, and
publish a GitHub Release with generated release notes.

## Package Contents

Portable archives include the app binary or `.app` bundle, `README.md`,
`README_EN.md`, and `LICENSE`.

Installers:

- **Windows** — Inno Setup (`packaging/windows/MarkdownMonkey.iss`) installs to
  Program Files (or a per-user folder) and can add a desktop shortcut.
- **Linux** — `dpkg -i MarkdownMonkey-<version>-linux-x64.deb` installs
  `/usr/bin/markdownmonkey` and a `.desktop` launcher.
- **macOS** — open the tarball and drag `MarkdownMonkey.app` to `/Applications`.

Project documentation also includes:

- [RELEASE.md](RELEASE.md) — this guide
- [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) — pre-tag acceptance checklist
- [screenshots/](screenshots/) — UI screenshots referenced by the READMEs (`main_zh.png`, `main_en.png`). Recapture after README or chrome changes so the in-app editor still matches the docs.
