# Vendor preview engines

Offline copies used by the desktop editor and preview pane.

| File | Version | License | Source |
|------|---------|---------|--------|
| `katex.min.js` / `katex.min.css` / `fonts/*.woff2` | 0.16.22 | MIT | [KaTeX](https://github.com/KaTeX/KaTeX) |
| `mermaid.min.js` | 11.4.1 | MIT | [Mermaid](https://github.com/mermaid-js/mermaid) |
| `codemirror6.bundle.js` | 6.x (`@codemirror/view` 6.43.11, `lang-markdown` 6.5.2) | MIT | [CodeMirror](https://github.com/codemirror/dev) |

HTML export loads KaTeX / Mermaid from jsDelivr so standalone files stay small.

Rebuild the CodeMirror IIFE after upgrading npm packages:

```powershell
cd scripts/codemirror
npm install
npm run build
```
