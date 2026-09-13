/**
 * CodeMirror 6 升级层：在 textarea 桥上挂接 EditorView
 * CodeMirror 6 upgrade: mount an EditorView on the textarea bridge
 */
(function () {
    if (window._mm_cmUpgradeInstalled) return;
    window._mm_cmUpgradeInstalled = true;
    window._mm_cmRetries = 0;

    var searchEffect = null;
    var searchField = null;

    function cmApi() {
        return window.MarkdownMonkeyCM || null;
    }

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

    /**
     * 清掉叠加层并隐藏回退 textarea
     * Strip overlays and hide the fallback textarea
     */
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
     * 挂载失败时恢复可见 textarea
     * Restore a visible textarea if the kernel fails to mount
     */
    function showFallback(ta) {
        if (ta) {
            ta.classList.remove('syntax-on');
            ta.removeAttribute('aria-hidden');
        }
        var host = document.querySelector('.editor-content');
        if (host) host.classList.remove('cm-on');
    }

    /**
     * 一次性创建搜索装饰字段
     * Create the search-decoration field once
     */
    function ensureSearchField() {
        var CM = cmApi();
        if (!CM || searchField) return;
        searchEffect = CM.StateEffect.define();
        searchField = CM.StateField.define({
            create: function () {
                return CM.Decoration.none;
            },
            update: function (deco, tr) {
                deco = deco.map(tr.changes);
                for (var i = 0; i < tr.effects.length; i++) {
                    if (tr.effects[i].is(searchEffect)) deco = tr.effects[i].value;
                }
                return deco;
            },
            provide: function (field) {
                return CM.EditorView.decorations.from(field);
            }
        });
    }

    /**
     * Enter：续 Markdown 列表 / 任务 / 引用，否则走默认换行缩进
     * Enter: continue Markdown lists / tasks / quotes, else default newline+indent
     */
    function markdownEnter(view) {
        var CM = cmApi();
        var state = view.state;
        if (state.selection.ranges.length !== 1 || !state.selection.main.empty) {
            return CM.insertNewlineAndIndent(view);
        }
        var from = state.selection.main.from;
        var line = state.doc.lineAt(from);
        var text = line.text;
        var indent = (text.match(/^\s*/) || [''])[0];
        var trimmed = text.trim();
        var prefix = '';
        var list = trimmed.match(/^([-*+])\s/);
        var ol = trimmed.match(/^(\d+)\.\s/);
        var task = trimmed.match(/^[-*+]\s\[[ xX]\]\s/);
        var quote = trimmed.match(/^>\s?/);
        if (task && trimmed.length > 6) {
            prefix = '- [ ] ';
        } else if (task) {
            view.dispatch({ changes: { from: line.from, to: from, insert: '' } });
            return true;
        } else if (list && trimmed.length > 2) {
            prefix = list[1] + ' ';
        } else if (list) {
            view.dispatch({ changes: { from: line.from, to: from, insert: '' } });
            return true;
        } else if (ol && trimmed.length > ol[0].length) {
            prefix = String(parseInt(ol[1], 10) + 1) + '. ';
        } else if (ol) {
            view.dispatch({ changes: { from: line.from, to: from, insert: '' } });
            return true;
        } else if (quote && trimmed.length > 1) {
            prefix = '> ';
        } else {
            return CM.insertNewlineAndIndent(view);
        }
        view.dispatch(state.replaceSelection('\n' + indent + prefix));
        return true;
    }

    /**
     * 销毁一个 EditorView 及其 host
     * Destroy one EditorView and its host
     */
    function destroyView(view) {
        if (!view) return;
        var host = view._mm_host;
        var ta = view._mm_ta;
        try {
            view.destroy();
        } catch (err) { /* ignore */ }
        if (ta && ta._mm_cm === view) ta._mm_cm = null;
        if (host && host.parentNode) host.remove();
        if (window._mm_cmInstance === view) window._mm_cmInstance = null;
    }

    /**
     * 拆掉 Dioxus 重绘后残留的旧实例
     * Tear down leftover hosts after a Dioxus patch
     */
    function destroyOrphanedHosts(keepHost) {
        var nodes = document.querySelectorAll('.editor-content .cm-host');
        for (var i = 0; i < nodes.length; i++) {
            if (keepHost && nodes[i] === keepHost) continue;
            var view = nodes[i]._mm_view;
            if (view) destroyView(view);
            else nodes[i].remove();
        }
        if (window._mm_cmInstance && window._mm_cmInstance._mm_host !== keepHost) {
            window._mm_cmInstance = keepHost && keepHost._mm_view ? keepHost._mm_view : null;
        }
    }

    /**
     * 把正文写回 textarea 并通知 Dioxus
     * Write the document back to the textarea and notify Dioxus
     */
    function syncTextarea(view, ta, fireInput) {
        if (!view || !ta) return;
        var next = view.state.doc.toString();
        if (ta.value !== next) ta.value = next;
        if (fireInput) {
            if (window._mm_cmLastSent === next) return;
            window._mm_cmLastSent = next;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        }
    }

    /**
     * 在光标处插入文本（粘贴/拖放图片）
     * Insert text at the cursor (image paste / drop)
     */
    function insertAtCursor(view, text) {
        view.dispatch(view.state.replaceSelection(text));
        view.focus();
    }

    /**
     * 绑定图片粘贴与拖放，行为与 textarea 回退一致
     * Bind image paste/drop so behavior matches the textarea fallback
     */
    function bindMediaInsert(view) {
        view.dom.addEventListener('paste', function (e) {
            var items = (e.clipboardData || {}).items;
            if (!items) return;
            for (var i = 0; i < items.length; i++) {
                if (items[i].type.indexOf('image/') === 0) {
                    e.preventDefault();
                    var blob = items[i].getAsFile();
                    var reader = new FileReader();
                    reader.onload = function (evt) {
                        var timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
                        insertAtCursor(view, '![' + 'image_' + timestamp + '](' + evt.target.result + ')');
                    };
                    reader.readAsDataURL(blob);
                    return;
                }
            }
        });
        view.dom.addEventListener('drop', function (e) {
            var files = e.dataTransfer && e.dataTransfer.files;
            if (!files || files.length === 0) return;
            for (var i = 0; i < files.length; i++) {
                var file = files[i];
                if (file.type.indexOf('image/') === 0) {
                    e.preventDefault();
                    e.stopPropagation();
                    var reader = new FileReader();
                    reader.onload = function (evt) {
                        var name = file.name.replace(/\.[^.]+$/, '');
                        insertAtCursor(view, '![' + name + '](' + evt.target.result + ')');
                    };
                    reader.readAsDataURL(file);
                    return;
                }
            }
        }, true);
        view.dom.addEventListener('keydown', function (e) {
            var mod = e.ctrlKey || e.metaKey;
            if (!mod) return;
            var key = e.key;
            if (key === 'z' || key === 'Z' || key === 'y' || key === 'Y') {
                e.preventDefault();
            }
        }, true);
    }

    /**
     * 创建并挂上 EditorView
     * Create and mount an EditorView
     */
    function mountView(ta) {
        var CM = cmApi();
        ensureSearchField();
        var content = document.querySelector('.editor-content');
        if (!content) return null;
        var host = document.createElement('div');
        host.className = 'cm-host';
        content.appendChild(host);

        var themeComp = new CM.Compartment();
        var highKeys = [{ key: 'Enter', run: markdownEnter }]
            .concat(CM.closeBracketsKeymap || [])
            .concat([CM.indentWithTab]);

        var view = new CM.EditorView({
            parent: host,
            state: CM.EditorState.create({
                doc: ta.value || '',
                extensions: [
                    CM.lineNumbers(),
                    CM.highlightActiveLine(),
                    CM.highlightActiveLineGutter(),
                    CM.drawSelection(),
                    CM.EditorView.lineWrapping,
                    CM.history(),
                    CM.markdown(),
                    CM.indentUnit.of('    '),
                    CM.closeBrackets(),
                    searchField,
                    themeComp.of(isDarkTheme() ? CM.darkTheme() : CM.lightTheme()),
                    CM.Prec.high(CM.keymap.of(highKeys)),
                    CM.keymap.of(CM.defaultKeymap),
                    CM.EditorView.updateListener.of(function (update) {
                        if (window._mm_cmApplyingExternal) return;
                        if (update.docChanged) {
                            syncTextarea(update.view, update.view._mm_ta, true);
                        }
                        if (update.selectionSet && update.view._mm_ta) {
                            update.view._mm_ta.dispatchEvent(new Event('select', { bubbles: true }));
                        }
                    })
                ]
            })
        });
        view._mm_ta = ta;
        view._mm_host = host;
        view._mm_theme = themeComp;
        host._mm_view = view;
        ta._mm_cm = view;
        bindMediaInsert(view);
        view.scrollDOM.addEventListener('scroll', function () {
            if (!window._mm_syncScrollEnabled) return;
            var el = view.scrollDOM;
            var max = el.scrollHeight - el.clientHeight;
            if (max <= 0) return;
            var preview = document.getElementById('preview-scroll');
            if (!preview) return;
            var pmax = preview.scrollHeight - preview.clientHeight;
            if (pmax > 0) preview.scrollTop = (el.scrollTop / max) * pmax;
        });
        return view;
    }

    /**
     * 把当前 textarea 升级为 CodeMirror 6
     * Upgrade the current textarea to CodeMirror 6
     */
    window._mm_upgradeToCodeMirror = function () {
        var ta = document.querySelector('.editor-textarea');
        if (!ta || window._mm_cmUpgrading) return;
        if (!cmApi()) {
            if (window._mm_cmRetries < 80) {
                window._mm_cmRetries += 1;
                setTimeout(window._mm_upgradeToCodeMirror, 80);
            } else {
                showFallback(ta);
            }
            return;
        }
        window._mm_cmRetries = 0;
        window._mm_cmUpgrading = true;
        try {
            if (window._mm_cmInstance && window._mm_cmInstance._mm_ta !== ta) {
                destroyView(window._mm_cmInstance);
            }
            destroyOrphanedHosts(ta._mm_cm && ta._mm_cm._mm_host ? ta._mm_cm._mm_host : null);
            if (ta._mm_cm && ta._mm_cm.dom && document.body.contains(ta._mm_cm.dom)) {
                window._mm_cmInstance = ta._mm_cm;
                hideOverlay(ta);
                installCmBridge();
                watchEditorHost();
                applyPendingEditorValue();
                window._mm_setCmTheme();
                return;
            }
            if (ta._mm_cm) {
                destroyView(ta._mm_cm);
            }
            var view = mountView(ta);
            if (!view) {
                showFallback(ta);
                return;
            }
            window._mm_cmInstance = view;
            hideOverlay(ta);
            installCmBridge();
            watchEditorHost();
            applyPendingEditorValue();
            window._mm_setCmTheme();
        } catch (err) {
            showFallback(ta);
        } finally {
            window._mm_cmUpgrading = false;
        }
    };

    /**
     * 套用内核挂载前缓存的正文
     * Apply text queued before the kernel mounted
     */
    function applyPendingEditorValue() {
        if (typeof window._mm_pendingEditorValue !== 'string') return;
        var pending = window._mm_pendingEditorValue;
        window._mm_pendingEditorValue = null;
        if (window._mm_setEditorValue) window._mm_setEditorValue(pending);
    }

    /**
     * 监视 Dioxus 补丁，避免旧实例和叠加层再次露出来
     * Watch Dioxus patches so leftover instances and overlays cannot reappear
     */
    function watchEditorHost() {
        var host = document.querySelector('.editor-content');
        if (!host || host._mm_cmObserver) return;
        host._mm_cmObserver = new MutationObserver(function () {
            if (window._mm_cmUpgrading) return;
            var extras = host.querySelectorAll('.cm-host');
            if (extras.length > 1) {
                destroyOrphanedHosts(window._mm_cmInstance && window._mm_cmInstance._mm_host);
            }
            var ta = host.querySelector('.editor-textarea');
            var live = window._mm_cmInstance &&
                window._mm_cmInstance.dom &&
                document.body.contains(window._mm_cmInstance.dom);
            if (live) {
                hideOverlay(ta);
            }
            if (ta && (!ta._mm_cm || !live) && cmApi()) {
                if (!live && extras.length > 0) {
                    destroyOrphanedHosts(null);
                }
                if (!host.querySelector('.cm-host')) {
                    window._mm_upgradeToCodeMirror();
                }
            }
        });
        host._mm_cmObserver.observe(host, { childList: true, subtree: true });
    }

    /**
     * 按应用主题切换 CodeMirror 配色
     * Switch the CodeMirror theme with the app theme
     */
    window._mm_setCmTheme = function () {
        var view = window._mm_cmInstance;
        var CM = cmApi();
        if (!view || !CM || !view._mm_theme) return;
        view.dispatch({
            effects: view._mm_theme.reconfigure(isDarkTheme() ? CM.darkTheme() : CM.lightTheme())
        });
    };

    function installCmBridge() {
        window._mm_setEditorValue = function (text) {
            if (typeof text !== 'string') text = String(text || '');
            if (window._mm_cmInstance) {
                var view = window._mm_cmInstance;
                var CM = cmApi();
                window._mm_cmApplyingExternal = true;
                try {
                    if (view.state.doc.toString() !== text) {
                        view.dispatch({
                            changes: { from: 0, to: view.state.doc.length, insert: text },
                            annotations: CM ? [CM.Transaction.addToHistory.of(false)] : []
                        });
                    }
                    syncTextarea(view, view._mm_ta || document.querySelector('.editor-textarea'), false);
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
                var view = window._mm_cmInstance;
                var CM = cmApi();
                window._mm_cmApplyingExternal = true;
                try {
                    if (view.state.doc.toString() !== text) {
                        view.dispatch({
                            changes: { from: 0, to: view.state.doc.length, insert: text },
                            annotations: CM ? [CM.Transaction.addToHistory.of(false)] : []
                        });
                    }
                    window._mm_cmLastSent = text;
                } finally {
                    window._mm_cmApplyingExternal = false;
                }
                var anchor = direction === 'backward' ? end : start;
                var head = direction === 'backward' ? start : end;
                view.dispatch({
                    selection: { anchor: anchor, head: head },
                    scrollIntoView: true
                });
                syncTextarea(view, view._mm_ta || document.querySelector('.editor-textarea'), false);
                view.focus();
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
                var view = window._mm_cmInstance;
                var value = view.state.doc.toString();
                var main = view.state.selection.main;
                return [
                    value,
                    utf16ToUtf8Offset(value, main.from),
                    utf16ToUtf8Offset(value, main.to),
                    main.head < main.anchor ? 'backward' : 'none'
                ];
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

        /**
         * 执行内核撤销/重做；没有实例时返回 false
         * Run kernel undo/redo; return false when no instance is mounted
         */
        window._mm_cmHistory = function (op) {
            var view = window._mm_cmInstance;
            var CM = cmApi();
            if (!view || !CM) return false;
            if (op === 'redo') CM.redo(view);
            else CM.undo(view);
            window._mm_cmLastSent = null;
            syncTextarea(view, view._mm_ta || document.querySelector('.editor-textarea'), true);
            return true;
        };

        window._mm_getEditorValue = function () {
            if (window._mm_cmInstance) return window._mm_cmInstance.state.doc.toString();
            var el = document.querySelector('.editor-textarea');
            return el ? el.value : '';
        };

        window._mm_getSelection = function () {
            var snap = window._mm_getEditorSnapshot();
            return [snap[1], snap[2]];
        };

        window._mm_scrollToLine = function (lineNumber) {
            if (window._mm_cmInstance) {
                var view = window._mm_cmInstance;
                var CM = cmApi();
                var lineNo = Math.max(1, Math.min((Number(lineNumber) || 0) + 1, view.state.doc.lines));
                var line = view.state.doc.line(lineNo);
                view.dispatch({
                    selection: { anchor: line.from },
                    effects: CM ? [CM.EditorView.scrollIntoView(line.from, { y: 'start', yMargin: 80 })] : [],
                    scrollIntoView: true
                });
                view.focus();
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
            var view = window._mm_cmInstance;
            var CM = cmApi();
            if (!view || !CM || !searchEffect) return;
            window._mm_searchActive = !!query;
            if (!query) {
                view.dispatch({ effects: searchEffect.of(CM.Decoration.none) });
                return;
            }
            var value = view.state.doc.toString();
            var hay = caseInsensitive ? value.toLowerCase() : value;
            var needle = caseInsensitive ? String(query).toLowerCase() : String(query);
            var from = 0;
            var idx = 0;
            var ranges = [];
            var currentFrom = -1;
            var currentTo = -1;
            while (needle && from <= hay.length) {
                var found = hay.indexOf(needle, from);
                if (found < 0) break;
                var to = found + query.length;
                var cls = idx === currentIndex ? 'search-highlight-current' : 'search-highlight';
                ranges.push(CM.Decoration.mark({ class: cls }).range(found, to));
                if (idx === currentIndex) {
                    currentFrom = found;
                    currentTo = to;
                }
                idx += 1;
                if (idx > 2000) break;
                from = found + Math.max(1, needle.length);
            }
            var effects = [searchEffect.of(CM.Decoration.set(ranges, true))];
            if (currentFrom >= 0) {
                effects.push(CM.EditorView.scrollIntoView(currentFrom, { y: 'center' }));
                view.dispatch({
                    selection: { anchor: currentFrom, head: currentTo },
                    effects: effects
                });
            } else {
                view.dispatch({ effects: effects });
            }
        };

        window._mm_reverseSyncScroll = function () {
            if (!window._mm_syncScrollEnabled) return;
            if (window._mm_cmInstance) {
                var preview = document.getElementById('preview-scroll');
                if (!preview) return;
                var psh = preview.scrollHeight - preview.clientHeight;
                if (psh <= 0) return;
                var el = window._mm_cmInstance.scrollDOM;
                var max = el.scrollHeight - el.clientHeight;
                if (max > 0) el.scrollTop = (preview.scrollTop / psh) * max;
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
