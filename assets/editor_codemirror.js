/**
 * CodeMirror 5 升级层：在现有 textarea 桥上挂接真正的编辑器内核
 * CodeMirror 5 upgrade: mount a real editor kernel on the existing textarea bridge
 */
(function () {
    if (window._mm_cmUpgradeInstalled) return;
    window._mm_cmUpgradeInstalled = true;
    window._mm_cmRetries = 0;

    function isDarkTheme() {
        var root = document.querySelector('.app-container') || document.documentElement;
        return (root.getAttribute('data-theme') || 'dark') !== 'light';
    }

    function utf16ToUtf8Offset(text, offset) {
        var safe = Math.max(0, Math.min(Number(offset) || 0, text.length));
        if (safe > 0 && safe < text.length) {
            var before = text.charCodeAt(safe - 1);
            var after = text.charCodeAt(safe);
            if (before >= 0xD800 && before <= 0xDBFF && after >= 0xDC00 && after <= 0xDFFF) {
                safe -= 1;
            }
        }
        return new TextEncoder().encode(text.slice(0, safe)).length;
    }

    function utf8ToUtf16Offset(text, byteOffset) {
        var target = Math.max(0, Number(byteOffset) || 0);
        var bytes = 0;
        var utf16 = 0;
        for (var ch of text) {
            var next = bytes + new TextEncoder().encode(ch).length;
            if (next > target) break;
            bytes = next;
            utf16 += ch.length;
        }
        return utf16;
    }

    function hideOverlay(ta) {
        document.querySelectorAll('.editor-highlight-overlay').forEach(function (el) {
            el.remove();
        });
        if (ta) {
            ta.classList.remove('syntax-on');
            ta.setAttribute('aria-hidden', 'true');
        }
        var host = document.querySelector('.editor-content');
        if (host) host.classList.add('cm-on');
    }

    /**
     * 拆掉 Dioxus 重绘后残留的旧 CodeMirror 实例
     * Tear down orphaned CodeMirror wrappers left after a Dioxus patch
     */
    function destroyOrphanedCodeMirrors(keep) {
        var keepEl = keep && keep.getWrapperElement ? keep.getWrapperElement() : null;
        var nodes = document.querySelectorAll('.editor-content .CodeMirror');
        for (var i = 0; i < nodes.length; i++) {
            var el = nodes[i];
            if (keepEl && el === keepEl) continue;
            var inst = el.CodeMirror;
            if (inst && typeof inst.toTextArea === 'function') {
                try {
                    inst.toTextArea();
                } catch (err) {
                    el.remove();
                }
            } else {
                el.remove();
            }
        }
        if (window._mm_cmInstance && window._mm_cmInstance !== keep) {
            window._mm_cmInstance = keep || null;
        }
    }

    /**
     * 把当前 textarea 升级为 CodeMirror
     * Upgrade the current textarea to CodeMirror
     */
    window._mm_upgradeToCodeMirror = function () {
        var ta = document.querySelector('.editor-textarea');
        if (!ta || window._mm_cmUpgrading) return;
        if (!window.CodeMirror) {
            if (window._mm_cmRetries < 40) {
                window._mm_cmRetries += 1;
                setTimeout(window._mm_upgradeToCodeMirror, 80);
            }
            return;
        }
        window._mm_cmRetries = 0;
        window._mm_cmUpgrading = true;
        try {
        if (window._mm_cmInstance && window._mm_cmInstance.getTextArea() !== ta) {
            try {
                var oldTa = window._mm_cmInstance.getTextArea();
                window._mm_cmInstance.toTextArea();
                if (oldTa) oldTa._mm_cm = null;
            } catch (err) { /* ignore */ }
            window._mm_cmInstance = null;
        }
        destroyOrphanedCodeMirrors(ta._mm_cm || null);
        if (ta._mm_cm) {
            window._mm_cmInstance = ta._mm_cm;
            hideOverlay(ta);
            installCmBridge();
            watchEditorHost();
            return;
        }

        var cm = window.CodeMirror.fromTextArea(ta, {
            mode: 'markdown',
            lineNumbers: true,
            lineWrapping: true,
            tabSize: 4,
            indentUnit: 4,
            indentWithTabs: false,
            theme: isDarkTheme() ? 'material-darker' : 'default',
            extraKeys: {
                Tab: function (editor) {
                    if (editor.somethingSelected()) editor.indentSelection('add');
                    else editor.replaceSelection('    ', 'end');
                },
                'Shift-Tab': function (editor) {
                    editor.indentSelection('subtract');
                },
                Enter: function (editor) {
                    var cursor = editor.getCursor();
                    var line = editor.getLine(cursor.line) || '';
                    var indent = (line.match(/^\s*/) || [''])[0];
                    var trimmed = line.trim();
                    var prefix = '';
                    var list = trimmed.match(/^([-*+])\s/);
                    var ol = trimmed.match(/^(\d+)\.\s/);
                    var task = trimmed.match(/^[-*+]\s\[[ xX]\]\s/);
                    var quote = trimmed.match(/^>\s?/);
                    if (task && trimmed.length > 6) prefix = '- [ ] ';
                    else if (task) {
                        editor.replaceRange('', { line: cursor.line, ch: 0 }, cursor);
                        return;
                    } else if (list && trimmed.length > 2) prefix = list[1] + ' ';
                    else if (list) {
                        editor.replaceRange('', { line: cursor.line, ch: 0 }, cursor);
                        return;
                    } else if (ol && trimmed.length > ol[0].length) {
                        prefix = (parseInt(ol[1], 10) + 1) + '. ';
                    } else if (ol) {
                        editor.replaceRange('', { line: cursor.line, ch: 0 }, cursor);
                        return;
                    } else if (quote && trimmed.length > 1) prefix = '> ';
                    editor.replaceSelection('\n' + indent + prefix, 'end');
                }
            }
        });
        ta._mm_cm = cm;
        window._mm_cmInstance = cm;
        hideOverlay(ta);
        document.querySelector('.editor-content') &&
            document.querySelector('.editor-content').classList.add('cm-on');

        cm.on('change', function () {
            if (window._mm_cmApplyingExternal) return;
            var next = cm.getValue();
            if (ta.value !== next) cm.save();
            if (window._mm_cmLastSent === next) return;
            window._mm_cmLastSent = next;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        });
        cm.on('cursorActivity', function () {
            ta.dispatchEvent(new Event('select', { bubbles: true }));
        });
        cm.on('scroll', function () {
            if (!window._mm_syncScrollEnabled) return;
            var info = cm.getScrollInfo();
            var max = info.height - info.clientHeight;
            if (max <= 0) return;
            var ratio = info.top / max;
            var el = document.getElementById('preview-scroll');
            if (el) {
                var pmax = el.scrollHeight - el.clientHeight;
                if (pmax > 0) el.scrollTop = ratio * pmax;
            }
        });
        installCmBridge();
        watchEditorHost();
        } finally {
            window._mm_cmUpgrading = false;
        }
    };

    /**
     * 监视 Dioxus 补丁，避免旧实例和叠加层再次露出来
     * Watch Dioxus patches so leftover instances and overlays cannot reappear
     */
    function watchEditorHost() {
        var host = document.querySelector('.editor-content');
        if (!host || host._mm_cmObserver) return;
        host._mm_cmObserver = new MutationObserver(function () {
            if (window._mm_cmUpgrading) return;
            var extras = host.querySelectorAll('.CodeMirror');
            if (extras.length > 1) {
                destroyOrphanedCodeMirrors(window._mm_cmInstance);
            }
            hideOverlay(host.querySelector('.editor-textarea'));
            var ta = host.querySelector('.editor-textarea');
            if (ta && !ta._mm_cm && extras.length === 0 && window.CodeMirror) {
                window._mm_upgradeToCodeMirror();
            }
        });
        host._mm_cmObserver.observe(host, { childList: true, subtree: true });
    }

    /**
     * 按主题切换 CodeMirror 配色
     * Switch the CodeMirror theme with the app theme
     */
    window._mm_setCmTheme = function () {
        if (!window._mm_cmInstance) return;
        window._mm_cmInstance.setOption('theme', isDarkTheme() ? 'material-darker' : 'default');
    };

    function installCmBridge() {
        window._mm_setEditorValue = function (text) {
            if (typeof text !== 'string') text = String(text || '');
            if (window._mm_cmInstance) {
                window._mm_cmApplyingExternal = true;
                try {
                    if (window._mm_cmInstance.getValue() !== text) {
                        window._mm_cmInstance.setValue(text);
                    }
                    window._mm_cmInstance.save();
                    window._mm_cmLastSent = text;
                } finally {
                    window._mm_cmApplyingExternal = false;
                }
                return;
            }
            var el = document.querySelector('.editor-textarea');
            if (el) el.value = text;
        };

        window._mm_setEditorState = function (text, byteStart, byteEnd, direction) {
            if (typeof text !== 'string') text = String(text || '');
            var start = utf8ToUtf16Offset(text, byteStart);
            var end = utf8ToUtf16Offset(text, byteEnd);
            if (window._mm_cmInstance) {
                window._mm_cmApplyingExternal = true;
                try {
                    if (window._mm_cmInstance.getValue() !== text) {
                        window._mm_cmInstance.setValue(text);
                    }
                    window._mm_cmLastSent = text;
                } finally {
                    window._mm_cmApplyingExternal = false;
                }
                var from = window._mm_cmInstance.posFromIndex(start);
                var to = window._mm_cmInstance.posFromIndex(end);
                if (direction === 'backward') {
                    window._mm_cmInstance.setSelection(to, from);
                } else {
                    window._mm_cmInstance.setSelection(from, to);
                }
                window._mm_cmInstance.save();
                window._mm_cmInstance.focus();
                return;
            }
            var el = document.querySelector('.editor-textarea');
            if (!el) return;
            el.value = text;
            try {
                el.setSelectionRange(start, end, direction === 'backward' ? 'backward' : 'forward');
            } catch (err) { /* ignore */ }
        };

        window._mm_getEditorSnapshot = function () {
            if (window._mm_cmInstance) {
                var value = window._mm_cmInstance.getValue();
                var from = window._mm_cmInstance.indexFromPos(window._mm_cmInstance.getCursor('from'));
                var to = window._mm_cmInstance.indexFromPos(window._mm_cmInstance.getCursor('to'));
                return [value, utf16ToUtf8Offset(value, from), utf16ToUtf8Offset(value, to), 'none'];
            }
            var el = document.querySelector('.editor-textarea');
            if (!el) return ['', 0, 0, 'none'];
            var text = el.value || '';
            return [
                text,
                utf16ToUtf8Offset(text, el.selectionStart || 0),
                utf16ToUtf8Offset(text, el.selectionEnd || 0),
                el.selectionDirection || 'none'
            ];
        };

        window._mm_getEditorValue = function () {
            if (window._mm_cmInstance) return window._mm_cmInstance.getValue();
            var el = document.querySelector('.editor-textarea');
            return el ? el.value : '';
        };

        window._mm_getSelection = function () {
            var snap = window._mm_getEditorSnapshot();
            return [snap[1], snap[2]];
        };

        window._mm_scrollToLine = function (lineNumber) {
            if (window._mm_cmInstance) {
                var line = Math.max(0, Number(lineNumber) || 0);
                window._mm_cmInstance.setCursor({ line: line, ch: 0 });
                window._mm_cmInstance.focus();
                var height = window._mm_cmInstance.defaultTextHeight() || 22;
                window._mm_cmInstance.scrollTo(null, Math.max(0, line * height - 80));
                return;
            }
            var ta = document.querySelector('.editor-textarea');
            if (!ta) return;
            var lines = (ta.value || '').split('\n');
            var pos = 0;
            for (var i = 0; i < Math.min(lineNumber, lines.length); i++) {
                pos += lines[i].length + 1;
            }
            ta.focus();
            ta.setSelectionRange(pos, pos);
            ta.scrollTop = lineNumber * 22.4 - ta.clientHeight / 2;
        };

        window._mm_highlightSearch = function (query, caseInsensitive, currentIndex) {
            var cm = window._mm_cmInstance;
            if (!cm) return;
            if (window._mm_cmSearchMarks) {
                for (var i = 0; i < window._mm_cmSearchMarks.length; i++) {
                    window._mm_cmSearchMarks[i].clear();
                }
            }
            window._mm_cmSearchMarks = [];
            window._mm_searchActive = !!query;
            if (!query) return;
            var value = cm.getValue();
            var hay = caseInsensitive ? value.toLowerCase() : value;
            var needle = caseInsensitive ? String(query).toLowerCase() : String(query);
            var from = 0;
            var idx = 0;
            while (needle && from <= hay.length) {
                var found = hay.indexOf(needle, from);
                if (found < 0) break;
                var fromPos = cm.posFromIndex(found);
                var toPos = cm.posFromIndex(found + query.length);
                var cls = idx === currentIndex ? 'search-highlight-current' : 'search-highlight';
                window._mm_cmSearchMarks.push(cm.markText(fromPos, toPos, { className: cls }));
                if (idx === currentIndex) {
                    cm.setSelection(fromPos, toPos);
                    cm.scrollIntoView({ from: fromPos, to: toPos }, 40);
                }
                idx += 1;
                if (idx > 2000) break;
                from = found + Math.max(1, needle.length);
            }
        };

        window._mm_reverseSyncScroll = function () {
            if (!window._mm_syncScrollEnabled) return;
            if (window._mm_cmInstance) {
                var preview = document.getElementById('preview-scroll');
                if (!preview) return;
                var psh = preview.scrollHeight - preview.clientHeight;
                if (psh <= 0) return;
                var info = window._mm_cmInstance.getScrollInfo();
                var max = info.height - info.clientHeight;
                if (max > 0) window._mm_cmInstance.scrollTo(null, (preview.scrollTop / psh) * max);
                return;
            }
            var previewEl = document.getElementById('preview-scroll');
            var ta = document.querySelector('.editor-textarea');
            if (!previewEl || !ta) return;
            var pmax = previewEl.scrollHeight - previewEl.clientHeight;
            var tmax = ta.scrollHeight - ta.clientHeight;
            if (pmax > 0 && tmax > 0) ta.scrollTop = (previewEl.scrollTop / pmax) * tmax;
        };
    }
})();
