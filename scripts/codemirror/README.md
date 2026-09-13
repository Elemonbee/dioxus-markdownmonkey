# CodeMirror 6 vendor bundle

Offline IIFE used by the desktop editor. `cargo build` does **not** run npm.

```powershell
cd scripts/codemirror
npm install
npm run build
```

The build writes `assets/vendor/codemirror6.bundle.js` and exposes `window.MarkdownMonkeyCM`.
