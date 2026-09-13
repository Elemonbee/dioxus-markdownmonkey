/**
 * CodeMirror 6 内核入口：打成 IIFE 后挂到 window.MarkdownMonkeyCM
 * CodeMirror 6 kernel entry: the IIFE exposes window.MarkdownMonkeyCM
 */
import {
    closeBrackets,
    closeBracketsKeymap,
} from "@codemirror/autocomplete";
import {
    defaultKeymap,
    history,
    indentWithTab,
    insertNewlineAndIndent,
    redo,
    undo,
} from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import {
    defaultHighlightStyle,
    HighlightStyle,
    indentUnit,
    syntaxHighlighting,
} from "@codemirror/language";
import {
    Compartment,
    EditorSelection,
    EditorState,
    Prec,
    StateEffect,
    StateField,
    Transaction,
} from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";
import {
    Decoration,
    drawSelection,
    EditorView,
    highlightActiveLine,
    highlightActiveLineGutter,
    keymap,
    lineNumbers,
} from "@codemirror/view";
import { tags } from "@lezer/highlight";

/**
 * 浅色主题：跟应用 CSS 变量走
 * Light theme that follows the app CSS variables
 */
function lightTheme() {
    return [
        EditorView.theme(
            {
                "&": {
                    color: "var(--text-primary)",
                    backgroundColor: "var(--bg-primary)",
                    height: "100%",
                },
                "&.cm-focused": { outline: "none" },
                ".cm-content": {
                    caretColor: "var(--text-primary)",
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--font-size-base)",
                    lineHeight: "1.6",
                },
                ".cm-gutters": {
                    backgroundColor: "var(--bg-secondary)",
                    color: "var(--text-muted)",
                    borderRight: "1px solid var(--border-color)",
                },
                ".cm-activeLine": {
                    backgroundColor:
                        "color-mix(in srgb, var(--bg-tertiary) 70%, transparent)",
                },
                ".cm-activeLineGutter": {
                    backgroundColor:
                        "color-mix(in srgb, var(--bg-tertiary) 70%, transparent)",
                },
                ".cm-scroller": {
                    overflow: "auto",
                    fontFamily: "var(--font-mono)",
                },
            },
            { dark: false },
        ),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    ];
}

/**
 * 深色主题：one-dark 打底，再用 CSS 变量盖过背景/行号，避免正文看不见
 * Dark theme: one-dark plus CSS-variable overrides so text stays visible
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
    ]);
    return [
        oneDark,
        syntaxHighlighting(headingStyle),
        EditorView.theme(
            {
                "&": {
                    color: "var(--text-primary)",
                    backgroundColor: "var(--bg-primary)",
                    height: "100%",
                },
                "&.cm-focused": { outline: "none" },
                ".cm-content": {
                    caretColor: "var(--text-primary)",
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--font-size-base)",
                    lineHeight: "1.6",
                    color: "var(--text-primary)",
                },
                ".cm-gutters": {
                    backgroundColor: "var(--bg-secondary)",
                    color: "var(--text-muted)",
                    borderRight: "1px solid var(--border-color)",
                },
                ".cm-activeLine": {
                    backgroundColor:
                        "color-mix(in srgb, var(--bg-tertiary) 55%, transparent)",
                },
                ".cm-scroller": {
                    overflow: "auto",
                    fontFamily: "var(--font-mono)",
                },
            },
            { dark: true },
        ),
    ];
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
    drawSelection,
    Decoration,
    defaultKeymap,
    history,
    indentWithTab,
    undo,
    redo,
    insertNewlineAndIndent,
    closeBrackets,
    closeBracketsKeymap,
    markdown,
    indentUnit,
    syntaxHighlighting,
    defaultHighlightStyle,
    HighlightStyle,
    tags,
    oneDark,
    lightTheme,
    darkTheme,
};
