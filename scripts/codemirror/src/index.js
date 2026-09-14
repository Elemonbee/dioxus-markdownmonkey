/**
 * CodeMirror 6 内核入口：打成 IIFE 后挂到 window.MarkdownMonkeyCM
 * CodeMirror 6 kernel entry: the IIFE exposes window.MarkdownMonkeyCM
 */
import {
    autocompletion,
    closeBrackets,
    closeBracketsKeymap,
    completionKeymap,
} from "@codemirror/autocomplete";
import {
    defaultKeymap,
    history,
    indentWithTab,
    insertNewlineAndIndent,
    redo,
    undo,
} from "@codemirror/commands";
import { css } from "@codemirror/lang-css";
import { html } from "@codemirror/lang-html";
import { javascript } from "@codemirror/lang-javascript";
import { json } from "@codemirror/lang-json";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { python } from "@codemirror/lang-python";
import { rust } from "@codemirror/lang-rust";
import { xml } from "@codemirror/lang-xml";
import {
    bracketMatching,
    defaultHighlightStyle,
    foldGutter,
    foldKeymap,
    HighlightStyle,
    indentOnInput,
    indentUnit,
    syntaxHighlighting,
    syntaxTree,
} from "@codemirror/language";
import { highlightSelectionMatches } from "@codemirror/search";
import { linter, lintGutter } from "@codemirror/lint";
import {
    Compartment,
    EditorSelection,
    EditorState,
    Prec,
    StateEffect,
    StateField,
    Transaction,
} from "@codemirror/state";
import {
    Decoration,
    drawSelection,
    dropCursor,
    EditorView,
    highlightActiveLine,
    highlightActiveLineGutter,
    highlightSpecialChars,
    hoverTooltip,
    keymap,
    lineNumbers,
    rectangularSelection,
    ViewPlugin,
    WidgetType,
} from "@codemirror/view";
import { tags } from "@lezer/highlight";

const editorFonts =
    "var(--font-mono), 'Microsoft YaHei', 'PingFang SC', sans-serif";

const sharedThemeRules = {
    "&": {
        color: "var(--text-primary)",
        backgroundColor: "var(--bg-primary)",
        height: "100%",
    },
    "&.cm-focused": { outline: "none" },
    ".cm-content": {
        caretColor: "var(--text-primary)",
        fontFamily: editorFonts,
        fontSize: "var(--font-size-base)",
        lineHeight: "1.6",
        fontVariantLigatures: "none",
    },
    ".cm-gutters": {
        backgroundColor: "var(--bg-secondary)",
        color: "var(--text-muted)",
        borderRight: "1px solid var(--border-color)",
    },
    ".cm-activeLine": {
        backgroundColor: "color-mix(in srgb, var(--bg-tertiary) 70%, transparent)",
    },
    ".cm-activeLineGutter": {
        backgroundColor: "color-mix(in srgb, var(--bg-tertiary) 70%, transparent)",
    },
    ".cm-scroller": {
        overflow: "auto",
        fontFamily: "var(--font-mono)",
    },
    ".cm-foldGutter .cm-gutterElement": {
        padding: "0 2px",
        cursor: "pointer",
        color: "var(--text-muted)",
    },
    ".cm-matchingBracket": {
        outline: "1px solid var(--accent-primary)",
        backgroundColor: "color-mix(in srgb, var(--accent-primary) 18%, transparent)",
    },
    ".cm-nonmatchingBracket": {
        outline: "1px solid var(--accent-error, #f38ba8)",
    },
    ".cm-foldPlaceholder": {
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-color)",
        color: "var(--text-muted)",
        margin: "0 4px",
        padding: "0 6px",
        borderRadius: "4px",
    },
    ".cm-tooltip": {
        backgroundColor: "var(--bg-secondary)",
        color: "var(--text-primary)",
        border: "1px solid var(--border-color)",
        fontFamily: editorFonts,
    },
    ".cm-tooltip-autocomplete ul li[aria-selected]": {
        backgroundColor: "color-mix(in srgb, var(--accent-primary) 28%, transparent)",
    },
    ".cm-task-checkbox": {
        margin: "0 4px 0 0",
        verticalAlign: "middle",
        cursor: "pointer",
    },
};

/**
 * 浅色主题：跟应用 CSS 变量走
 * Light theme that follows the app CSS variables
 */
function lightTheme() {
    return [
        EditorView.theme(sharedThemeRules, { dark: false }),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    ];
}

/**
 * 深色主题：只用 CSS 变量，避免 one-dark 字体栈把中文画成乱码
 * Dark theme: CSS variables only; skip one-dark so its font stack cannot scramble CJK
 */
function darkTheme() {
    const headingStyle = HighlightStyle.define([
        { tag: tags.heading, color: "var(--syntax-heading, #89b4fa)", fontWeight: "600" },
        { tag: tags.comment, color: "var(--syntax-comment, #6c7086)" },
        { tag: tags.link, color: "var(--syntax-function, #74c7ec)" },
        { tag: tags.url, color: "var(--syntax-function, #74c7ec)" },
        { tag: tags.emphasis, fontStyle: "italic" },
        { tag: tags.strong, fontWeight: "600" },
        { tag: tags.strikethrough, textDecoration: "line-through" },
        { tag: tags.monospace, color: "var(--syntax-string, #a6e3a1)" },
        { tag: tags.keyword, color: "var(--syntax-keyword, #cba6f7)" },
        { tag: tags.string, color: "var(--syntax-string, #a6e3a1)" },
        { tag: tags.number, color: "var(--syntax-number, #fab387)" },
        { tag: tags.bool, color: "var(--syntax-number, #fab387)" },
        { tag: tags.variableName, color: "var(--text-primary)" },
        { tag: tags.typeName, color: "var(--syntax-function, #74c7ec)" },
    ]);
    return [
        syntaxHighlighting(headingStyle, { fallback: true }),
        EditorView.theme(
            {
                ...sharedThemeRules,
                ".cm-content": {
                    ...sharedThemeRules[".cm-content"],
                    color: "var(--text-primary)",
                },
                ".cm-activeLine": {
                    backgroundColor:
                        "color-mix(in srgb, var(--bg-tertiary) 55%, transparent)",
                },
            },
            { dark: true },
        ),
    ];
}

/**
 * 围栏代码语言：只打包常用几种，避免 IIFE 过大
 * Fenced-code languages: keep the IIFE small by bundling a short list
 */
function codeLanguages(info) {
    const name = String(info || "")
        .trim()
        .split(/\s+/)[0]
        .toLowerCase();
    switch (name) {
        case "js":
        case "javascript":
        case "mjs":
        case "cjs":
        case "jsx":
            return javascript({ jsx: name === "jsx" }).language;
        case "ts":
        case "tsx":
        case "typescript":
            return javascript({ typescript: true, jsx: name === "tsx" }).language;
        case "json":
            return json().language;
        case "py":
        case "python":
            return python().language;
        case "rs":
        case "rust":
            return rust().language;
        case "css":
            return css().language;
        case "html":
        case "htm":
            return html().language;
        case "xml":
        case "svg":
            return xml().language;
        default:
            return null;
    }
}

/**
 * GFM Markdown + 围栏着色
 * GFM Markdown plus fenced-code highlighting
 */
function markdownSupport() {
    return markdown({
        base: markdownLanguage,
        codeLanguages,
        addKeymap: true,
    });
}

/**
 * 光标是否落在代码节点里（加粗等格式应跳过）
 * Whether the cursor sits inside a code node (bold etc. should skip)
 */
function isInCode(state, pos) {
    let node = syntaxTree(state).resolveInner(pos, 1);
    while (node) {
        const n = node.name;
        if (
            n === "FencedCode" ||
            n === "CodeBlock" ||
            n === "InlineCode" ||
            n === "Comment"
        ) {
            return true;
        }
        node = node.parent;
    }
    return false;
}

/**
 * 切换包裹标记；已包裹则拆掉
 * Toggle wrap markers; unwrap when already wrapped
 */
function wrapToggle(view, prefix, suffix, placeholder, skipCode) {
    const state = view.state;
    const sel = state.selection.main;
    const from = sel.from;
    const to = sel.to;
    if (skipCode && isInCode(state, from)) return true;
    let inner = state.sliceDoc(from, to);
    const wrapped =
        inner.length >= prefix.length + suffix.length &&
        inner.startsWith(prefix) &&
        inner.endsWith(suffix) &&
        (prefix !== "*" || !inner.startsWith("**"));
    if (wrapped) {
        inner = inner.slice(prefix.length, inner.length - suffix.length);
        view.dispatch({
            changes: { from, to, insert: inner },
            selection: EditorSelection.range(from, from + inner.length),
        });
        return true;
    }
    const before = state.sliceDoc(Math.max(0, from - prefix.length), from);
    const after = state.sliceDoc(to, Math.min(state.doc.length, to + suffix.length));
    if (
        before === prefix &&
        after === suffix &&
        (prefix !== "*" || state.sliceDoc(Math.max(0, from - 2), from) !== "**")
    ) {
        view.dispatch({
            changes: [
                { from: from - prefix.length, to: from, insert: "" },
                { from: to, to: to + suffix.length, insert: "" },
            ],
            selection: EditorSelection.range(from - prefix.length, to - prefix.length),
        });
        return true;
    }
    const body = inner || placeholder || "Text";
    view.dispatch({
        changes: { from, to, insert: prefix + body + suffix },
        selection: EditorSelection.range(
            from + prefix.length,
            from + prefix.length + body.length,
        ),
    });
    return true;
}

/**
 * 设置或取消 ATX 标题级别
 * Set or clear an ATX heading level
 */
function setHeading(view, level) {
    const line = view.state.doc.lineAt(view.state.selection.main.from);
    const match = line.text.match(/^(#{1,6})\s+/);
    const want = `${"#".repeat(level)} `;
    if (match && match[1].length === level) {
        view.dispatch({
            changes: { from: line.from, to: line.from + match[0].length, insert: "" },
        });
        return true;
    }
    const to = match ? line.from + match[0].length : line.from;
    view.dispatch({
        changes: { from: line.from, to, insert: want },
    });
    return true;
}

/**
 * 切换行首前缀（列表 / 引用）
 * Toggle a line-start prefix (lists / quotes)
 */
function toggleLinePrefix(view, prefix) {
    const line = view.state.doc.lineAt(view.state.selection.main.from);
    if (line.text.startsWith(prefix)) {
        view.dispatch({
            changes: { from: line.from, to: line.from + prefix.length, insert: "" },
        });
    } else {
        view.dispatch({
            changes: { from: line.from, insert: prefix },
        });
    }
    return true;
}

/**
 * 在内核里执行工具栏格式化，写入一条可撤销事务
 * Apply a toolbar format in-kernel as one undoable transaction
 */
function applyMarkdownFormat(view, kind, placeholder) {
    const ph = placeholder || "Text";
    switch (kind) {
        case "bold":
            return wrapToggle(view, "**", "**", ph, true);
        case "italic":
            return wrapToggle(view, "*", "*", ph, true);
        case "code":
            return wrapToggle(view, "`", "`", ph, false);
        case "link":
            return wrapToggle(view, "[", "](url)", ph, true);
        case "codeblock":
            return wrapToggle(view, "```\n", "\n```\n", ph, false);
        case "h1":
            return setHeading(view, 1);
        case "h2":
            return setHeading(view, 2);
        case "h3":
            return setHeading(view, 3);
        case "bullet":
            return toggleLinePrefix(view, "- ");
        case "numbered":
            return toggleLinePrefix(view, "1. ");
        case "quote":
            return toggleLinePrefix(view, "> ");
        case "hr":
            view.dispatch(view.state.replaceSelection("\n---\n"));
            return true;
        default:
            return false;
    }
}

/**
 * 任务列表复选框：替换 `[ ]` / `[x]`，点击改源码
 * Task-list checkboxes: replace `[ ]` / `[x]`; clicks rewrite the source
 */
class TaskCheckboxWidget extends WidgetType {
    constructor(from, checked) {
        super();
        this.from = from;
        this.checked = checked;
    }

    eq(other) {
        return other.from === this.from && other.checked === this.checked;
    }

    toDOM(view) {
        const input = document.createElement("input");
        input.type = "checkbox";
        input.checked = this.checked;
        input.className = "cm-task-checkbox";
        input.tabIndex = -1;
        input.setAttribute("aria-label", this.checked ? "task done" : "task");
        input.addEventListener("mousedown", (event) => event.preventDefault());
        input.addEventListener("change", () => {
            const mark = input.checked ? "x" : " ";
            view.dispatch({
                changes: { from: this.from, to: this.from + 1, insert: mark },
            });
        });
        return input;
    }

    ignoreEvent() {
        return false;
    }
}

function taskDecorations(view) {
    const builder = [];
    let inFence = false;
    for (let number = 1; number <= view.state.doc.lines; number += 1) {
        const line = view.state.doc.line(number);
        const trimmed = line.text.trimStart();
        if (trimmed.startsWith("```") || trimmed.startsWith("~~~")) {
            inFence = !inFence;
            continue;
        }
        if (inFence) continue;
        const match = line.text.match(/^(\s*[-*+]\s+)\[([ xX])\](?=\s|$)/);
        if (!match) continue;
        const bracketAt = line.from + match[1].length;
        const markAt = bracketAt + 1;
        const checked = match[2] !== " ";
        builder.push(
            Decoration.replace({
                widget: new TaskCheckboxWidget(markAt, checked),
            }).range(bracketAt, bracketAt + 3),
        );
    }
    return Decoration.set(builder, true);
}

function taskCheckbox() {
    return ViewPlugin.fromClass(
        class {
            constructor(view) {
                this.decorations = taskDecorations(view);
            }

            update(update) {
                if (update.docChanged) this.decorations = taskDecorations(update.view);
            }
        },
        { decorations: (plugin) => plugin.decorations },
    );
}

/**
 * `/table` 片段、`](#` 标题锚点、`](` 工作区文件
 * `/table` snippets, `](#` heading anchors, `](` workspace files
 */
function markdownCompletions(context) {
    const slash = context.matchBefore(/(?<=^|[\s])\/[A-Za-z]*$/);
    if (slash) {
        const query = slash.text.slice(1).toLowerCase();
        const snippets = [
            {
                label: "/table",
                displayLabel: "table",
                type: "snippet",
                detail: "Markdown table",
                apply: "| Column | Column |\n| --- | --- |\n|  |  |",
            },
            {
                label: "/task",
                displayLabel: "task",
                type: "snippet",
                detail: "Task item",
                apply: "- [ ] ",
            },
            {
                label: "/mermaid",
                displayLabel: "mermaid",
                type: "snippet",
                apply: "```mermaid\nflowchart LR\n  A --> B\n```",
            },
            {
                label: "/katex",
                displayLabel: "katex",
                type: "snippet",
                apply: "$$\n\n$$",
            },
            {
                label: "/code",
                displayLabel: "code",
                type: "snippet",
                apply: "```\n\n```",
            },
        ];
        return {
            from: slash.from,
            options: snippets.filter(
                (item) =>
                    !query ||
                    item.displayLabel.startsWith(query) ||
                    item.label.slice(1).startsWith(query),
            ),
            validFor: /^\/[A-Za-z]*$/,
        };
    }

    const headingLink = context.matchBefore(/\]\(#[^)\s]*$/);
    if (headingLink) {
        const options = [];
        const seen = new Set();
        const headingRe = /^(#{1,6})\s+(.+)$/gm;
        const doc = context.state.doc.toString();
        let match = headingRe.exec(doc);
        while (match) {
            const title = match[2].trim();
            const slug = title
                .toLowerCase()
                .replace(/[^\p{L}\p{N}\s-]/gu, "")
                .trim()
                .replace(/\s+/g, "-");
            if (title && slug && !seen.has(slug)) {
                seen.add(slug);
                options.push({
                    label: `#${slug}`,
                    type: "text",
                    detail: title,
                    apply: `](#${slug}`,
                });
            }
            match = headingRe.exec(doc);
        }
        return { from: headingLink.from, options, validFor: /\]\(#[^)\s]*$/ };
    }

    const fileLink = context.matchBefore(/\]\([^)\s]*$/);
    if (fileLink) {
        const typed = fileLink.text.slice(2).toLowerCase();
        const files = Array.isArray(window._mm_workspaceFiles)
            ? window._mm_workspaceFiles
            : [];
        const options = files
            .slice(0, 200)
            .filter(
                (file) => !typed || String(file).toLowerCase().includes(typed),
            )
            .map((file) => {
                const name = String(file);
                const image = isImagePath(name);
                return {
                    label: name,
                    type: image ? "image" : "text",
                    detail: image ? "image" : "markdown",
                    apply: `](${name}`,
                };
            });
        if (!options.length) return null;
        return { from: fileLink.from, options, validFor: /\]\([^)\s]*$/ };
    }

    return null;
}

function markdownAutocompletion() {
    return [
        autocompletion({ activateOnTyping: true, icons: false }),
        markdownLanguage.data.of({ autocomplete: markdownCompletions }),
    ];
}

/**
 * 是否为图片相对路径 / Whether a relative path looks like an image
 */
function isImagePath(path) {
    return /\.(png|jpe?g|gif|webp|svg|bmp|ico)$/i.test(String(path || ""));
}

/**
 * 规范化工作区相对路径 / Normalize a workspace-relative path
 */
function normalizeWorkspacePath(path) {
    return String(path || "")
        .replace(/\\/g, "/")
        .replace(/^\.\//, "");
}

/**
 * 链接目标是否存在于工作区文件列表 / Whether a link target exists in the workspace file list
 */
function workspaceHasPath(url) {
    const files = Array.isArray(window._mm_workspaceFiles)
        ? window._mm_workspaceFiles
        : [];
    if (!files.length) return null;
    const norm = normalizeWorkspacePath(url);
    const found = files.some((file) => {
        const item = normalizeWorkspacePath(file);
        return item === norm || item.endsWith("/" + norm);
    });
    return found;
}

/**
 * 外部或锚点链接不检查本地文件 / Skip local-file checks for external or anchor URLs
 */
function isExternalOrAnchor(url) {
    return /^(https?:|mailto:|#|data:)/i.test(String(url || "").trim());
}

/**
 * 悬停显示链接、图片与脚注
 * Hover tooltips for links, images, and footnotes
 */
function markdownHover() {
    return hoverTooltip((view, pos) => {
        const line = view.state.doc.lineAt(pos);
        const rel = pos - line.from;
        const text = line.text;
        const linkRe = /(!?)\[([^\]]*)\]\(([^)]*)\)/g;
        let match;
        while ((match = linkRe.exec(text))) {
            const start = match.index;
            const end = start + match[0].length;
            if (rel < start || rel > end) continue;
            const isImage = match[1] === "!";
            const alt = match[2] || "";
            const url = (match[3] || "").trim();
            return {
                pos: line.from + start,
                end: line.from + end,
                above: true,
                create() {
                    const dom = document.createElement("div");
                    const exists = isExternalOrAnchor(url)
                        ? true
                        : workspaceHasPath(url);
                    const kind = isImage ? "Image" : "Link";
                    let status = url || "(empty)";
                    if (!url) status = "Empty target";
                    else if (isExternalOrAnchor(url)) status = url;
                    else if (exists === false) status = `${url} — missing`;
                    else status = url;
                    dom.className =
                        exists === false || !url
                            ? "cm-mm-hover cm-mm-hover-missing"
                            : "cm-mm-hover";
                    dom.textContent = alt
                        ? `${kind}: ${status}\n${alt}`
                        : `${kind}: ${status}`;
                    return { dom };
                },
            };
        }

        const footnoteRe = /\[\^([^\]]+)\]/g;
        while ((match = footnoteRe.exec(text))) {
            const start = match.index;
            const end = start + match[0].length;
            if (rel < start || rel > end) continue;
            const id = match[1];
            const defRe = new RegExp(
                `^\\[\\^${id.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\]:\\s*(.*)$`,
                "m",
            );
            const def = defRe.exec(view.state.doc.toString());
            return {
                pos: line.from + start,
                end: line.from + end,
                above: true,
                create() {
                    const dom = document.createElement("div");
                    dom.className = def ? "cm-mm-hover" : "cm-mm-hover cm-mm-hover-missing";
                    dom.textContent = def
                        ? `Footnote: ${def[1] || "(empty)"}`
                        : `Footnote [^${id}] is undefined`;
                    return { dom };
                },
            };
        }
        return null;
    });
}

/**
 * 轻量 Markdown lint：未闭合围栏、空链接、缺失路径、标题跳级
 * Lightweight Markdown lint: unclosed fences, empty links, missing paths, heading skips
 */
function markdownDiagnostics(view) {
    const diagnostics = [];
    const doc = view.state.doc;
    if (doc.length > 200000) return diagnostics;
    const hasWorkspace = Array.isArray(window._mm_workspaceFiles)
        && window._mm_workspaceFiles.length > 0;

    let inFence = false;
    let fenceFrom = 0;
    let fenceTo = 0;
    let lastHeading = 0;
    for (let i = 1; i <= doc.lines; i++) {
        const line = doc.line(i);
        const text = line.text;
        if (/^(`{3,}|~{3,})/.test(text)) {
            if (!inFence) {
                inFence = true;
                fenceFrom = line.from;
                fenceTo = line.to;
            } else {
                inFence = false;
            }
            continue;
        }
        if (inFence) continue;

        const heading = text.match(/^(#{1,6})(?=\s|$)/);
        if (heading) {
            const level = heading[1].length;
            if (lastHeading && level > lastHeading + 1) {
                diagnostics.push({
                    from: line.from,
                    to: line.from + level,
                    severity: "info",
                    message: "Heading level skipped",
                });
            }
            lastHeading = level;
        }

        const linkRe = /(!?)\[([^\]]*)\]\(([^)]*)\)/g;
        let match;
        while ((match = linkRe.exec(text))) {
            const from = line.from + match.index;
            const to = from + match[0].length;
            const url = (match[3] || "").trim();
            if (!url) {
                diagnostics.push({
                    from,
                    to,
                    severity: "warning",
                    message: "Empty link target",
                });
                continue;
            }
            if (!hasWorkspace || isExternalOrAnchor(url)) continue;
            if (workspaceHasPath(url) === false) {
                diagnostics.push({
                    from,
                    to,
                    severity: "info",
                    message: "Path not found in workspace",
                });
            }
        }
    }
    if (inFence) {
        diagnostics.push({
            from: fenceFrom,
            to: fenceTo,
            severity: "warning",
            message: "Unclosed code fence",
        });
    }
    return diagnostics;
}

function markdownLint() {
    return [lintGutter(), linter(markdownDiagnostics, { delay: 750 })];
}

window.MarkdownMonkeyCM = {
    EditorView,
    EditorState,
    EditorSelection,
    Compartment,
    StateField,
    StateEffect,
    Prec,
    Transaction,
    keymap,
    lineNumbers,
    highlightActiveLine,
    highlightActiveLineGutter,
    highlightSpecialChars,
    drawSelection,
    dropCursor,
    rectangularSelection,
    Decoration,
    ViewPlugin,
    WidgetType,
    defaultKeymap,
    history,
    indentWithTab,
    undo,
    redo,
    insertNewlineAndIndent,
    closeBrackets,
    closeBracketsKeymap,
    completionKeymap,
    foldKeymap,
    markdown,
    markdownLanguage,
    markdownSupport,
    indentUnit,
    indentOnInput,
    syntaxHighlighting,
    syntaxTree,
    defaultHighlightStyle,
    HighlightStyle,
    tags,
    bracketMatching,
    foldGutter,
    highlightSelectionMatches,
    taskCheckbox,
    markdownAutocompletion,
    markdownHover,
    markdownLint,
    applyMarkdownFormat,
    lightTheme,
    darkTheme,
};
