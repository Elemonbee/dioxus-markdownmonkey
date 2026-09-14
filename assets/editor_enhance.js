                                                                                                                                           /**
 * MarkdownMonkey 编辑器增强脚本
 * Editor enhancement script
 * 
 * 功能/Features:
 * - Markdown 语法着色 / Markdown syntax coloring
 * - 搜索匹配高亮 / Search match highlighting
 * - Tab 键缩进/反缩进 / Tab indent/outdent
 * - Enter 自动缩进 + Markdown 列表续行 / Enter auto-indent + Markdown list continuation
 * - 括号/引号自动配对 / Bracket/quote auto-pairing
 * - 退格删除配对符号 / Backspace deletes paired symbols
 */
// 使用全局初始化函数，支持 Dioxus 重新创建 textarea 时重新附加增强功能
// Use global init function to support re-attaching when Dioxus recreates the textarea

// 同步滚动开关必须挂在 window 上：脚本重跑或 textarea 重建时不得重置用户选择
// Sync-scroll flag lives on window so script reload / textarea rebuild cannot reset it
if (typeof window._mm_syncScrollEnabled !== 'boolean') {
    window._mm_syncScrollEnabled = true;
}
window._mm_setSyncScroll = function(enabled) {
    window._mm_syncScrollEnabled = enabled === true || enabled === 1 || enabled === 'true';
};

window._mm_initEditor = function() {
    var ta = document.querySelector('.editor-textarea');
    if (!ta) return;
    // 叠加层会把 textarea 藏起来；内核未挂上时只剩“框选才看得见”
    // The overlay hides the textarea; if the kernel is not mounted, only a selection reveals text
    document.querySelectorAll('.editor-highlight-overlay').forEach(function (el) {
        el.remove();
    });
    ta.classList.remove('syntax-on');
    if (window._mm_cmInstance || document.querySelector('.editor-content .cm-host')) {
        ta._mm_enhanced = true;
        return;
    }
    // 如果已有增强且 textarea 未被替换，跳过
    // If already enhanced and textarea hasn't been replaced, skip
    if (ta._mm_enhanced) return;
    ta._mm_enhanced = true;

    // ========== UTF-16 / UTF-8 字节偏移桥接 / UTF-16 / UTF-8 byte-offset bridge ==========
    // DOM textarea 使用 UTF-16 code units；Rust 状态统一使用 UTF-8 byte offsets。
    // DOM textarea uses UTF-16 code units; Rust state consistently uses UTF-8 byte offsets.
    function clampUtf16Boundary(text, offset) {
        var safe = Math.max(0, Math.min(Number(offset) || 0, text.length));
        if (safe > 0 && safe < text.length) {
            var before = text.charCodeAt(safe - 1);
            var after = text.charCodeAt(safe);
            if (before >= 0xD800 && before <= 0xDBFF && after >= 0xDC00 && after <= 0xDFFF) {
                safe -= 1;
            }
        }
        return safe;
    }

    function utf16ToUtf8Offset(text, offset) {
        var safe = clampUtf16Boundary(text, offset);
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

    function editorSnapshot(el) {
        if (!el) return ['', 0, 0, 'none'];
        var value = el.value || '';
        return [
            value,
            utf16ToUtf8Offset(value, el.selectionStart || 0),
            utf16ToUtf8Offset(value, el.selectionEnd || 0),
            el.selectionDirection || 'none'
        ];
    }

    // 持续更新全局 UTF-8 字节选区，供 Rust 和 AI 选区功能读取。
    // Keep the global UTF-8 byte selection current for Rust and AI selection features.
    function syncSelection() {
        var snapshot = editorSnapshot(ta);
        window._mm_selStart = snapshot[1];
        window._mm_selEnd = snapshot[2];
    }
    syncSelection();
    ta.addEventListener('select', syncSelection);
    ta.addEventListener('keyup', syncSelection);
    ta.addEventListener('mouseup', syncSelection);
    ta.addEventListener('input', syncSelection);
    ta.addEventListener('click', syncSelection);
    window._mm_getSelection = function() {
        var el = document.querySelector('.editor-textarea');
        var snapshot = editorSnapshot(el);
        return [snapshot[1], snapshot[2]];
    };
    window._mm_getEditorSnapshot = function() {
        return editorSnapshot(document.querySelector('.editor-textarea'));
    };
    window._mm_getEditorValue = function() {
        var el = document.querySelector('.editor-textarea');
        return el ? el.value : '';
    };
    window._mm_setEditorValue = function(text) {
        var el = document.querySelector('.editor-textarea');
        if (!el) return;
        if (typeof text !== 'string') text = String(text || '');
        if (el.value !== text) {
            el.value = text;
        }
        if (window._mm_refreshSyntax) window._mm_refreshSyntax();
    };
    window._mm_setEditorState = function(text, byteStart, byteEnd, direction) {
        var el = document.querySelector('.editor-textarea');
        if (!el) return;
        if (typeof text !== 'string') text = String(text || '');
        var start = utf8ToUtf16Offset(text, byteStart);
        var end = utf8ToUtf16Offset(text, byteEnd);
        var selectionDirection = direction === 'backward' ? 'backward' : 'forward';
        var restore = function() {
            var current = document.querySelector('.editor-textarea');
            if (!current) return;
            if (current.value !== text) current.value = text;
            try {
                current.focus({ preventScroll: true });
            } catch (e) {
                current.focus();
            }
            current.setSelectionRange(start, end, selectionDirection);
            if (current._mm_enhanced) {
                var currentSnapshot = editorSnapshot(current);
                window._mm_selStart = currentSnapshot[1];
                window._mm_selEnd = currentSnapshot[2];
            }
        };
        restore();
        queueMicrotask(restore);
        requestAnimationFrame(restore);
        if (window._mm_refreshSyntax) window._mm_refreshSyntax();
    };
    
    /**
     * 清掉残留叠加层，保证回退 textarea 始终可见
     * Strip leftover overlays so the fallback textarea stays visible
     */
    window._mm_refreshSyntax = function() {
        ta.classList.remove('syntax-on');
        document.querySelectorAll('.editor-highlight-overlay').forEach(function (el) {
            el.remove();
        });
    };
    /**
     * 无内核时用原生选区定位搜索命中
     * Fall back to native selection when the editor kernel is not mounted
     */
    window._mm_highlightSearch = function(query, caseInsensitive, currentIndex) {
        if (!query) return;
        var content = ta.value || '';
        var hay = caseInsensitive ? content.toLowerCase() : content;
        var needle = caseInsensitive ? String(query).toLowerCase() : String(query);
        if (!needle) return;
        var from = 0;
        var idx = 0;
        while (from <= hay.length) {
            var found = hay.indexOf(needle, from);
            if (found < 0) break;
            if (idx === (currentIndex || 0)) {
                try { ta.focus({ preventScroll: true }); } catch (err) { ta.focus(); }
                ta.setSelectionRange(found, found + query.length);
                var lineNum = (content.substring(0, found).match(/\n/g) || []).length;
                var lineHeight = parseFloat(getComputedStyle(ta).lineHeight) || 22;
                ta.scrollTop = Math.max(0, lineNum * lineHeight - ta.clientHeight / 3);
                break;
            }
            idx += 1;
            from = found + Math.max(1, needle.length);
        }
    };

    // 同步滚动：在 JS 侧直接处理，避免经过 Dioxus 信号路由造成性能开销
    // Sync scroll: handled directly in JS to avoid Dioxus signal routing overhead
    var rafId = null;
    var lastRatio = -1;
    ta.addEventListener('scroll', function() {
        if (!window._mm_syncScrollEnabled) return;
        if (rafId !== null) return;
        rafId = requestAnimationFrame(function() {
            rafId = null;
            var sh = ta.scrollHeight - ta.clientHeight;
            if (sh <= 0) return;
            var ratio = ta.scrollTop / sh;
            if (Math.abs(ratio - lastRatio) < 0.002) return;
            lastRatio = ratio;
            var el = document.getElementById('preview-scroll');
            if (el) {
                var maxScroll = el.scrollHeight - el.clientHeight;
                if (maxScroll > 0) {
                    el.scrollTop = ratio * maxScroll;
                }
            }
        });
    });
    
    // ========== 键盘事件处理 / Keyboard Event Handling ==========
    ta.addEventListener('keydown', function(e) {
        
        // ---------- Tab 键：缩进/反缩进 / Tab: indent/outdent ----------
        if (e.key === 'Tab') {
            e.preventDefault();
            var start = ta.selectionStart;
            var end = ta.selectionEnd;
            var val = ta.value;
            
            if (start === end) {
                // 无选区：插入4个空格 / No selection: insert 4 spaces
                ta.value = val.substring(0, start) + '    ' + val.substring(end);
                ta.selectionStart = ta.selectionEnd = start + 4;
            } else {
                // 有选区：整块缩进/反缩进 / Has selection: indent/outdent block
                var lineStart = val.lastIndexOf('\n', start - 1) + 1;
                var lineEnd = val.indexOf('\n', end - 1);
                if (lineEnd === -1) lineEnd = val.length;
                var selected = val.substring(lineStart, lineEnd);
                var lines = selected.split('\n');
                if (!e.shiftKey) {
                    var indented = lines.map(function(l) { return '    ' + l; }).join('\n');
                    ta.value = val.substring(0, lineStart) + indented + val.substring(lineEnd);
                    ta.selectionStart = start + 4;
                    ta.selectionEnd = end + 4 * lines.length;
                } else {
                    var outdented = lines.map(function(l) {
                        if (l.startsWith('    ')) return l.substring(4);
                        if (l.startsWith('\t')) return l.substring(1);
                        return l;
                    }).join('\n');
                    ta.value = val.substring(0, lineStart) + outdented + val.substring(lineEnd);
                    var reduction = selected.length - outdented.length;
                    ta.selectionStart = Math.max(lineStart, start - 4);
                    ta.selectionEnd = Math.max(lineStart, end - reduction);
                }
            }
            ta.dispatchEvent(new Event('input', { bubbles: true }));
            return;
        }
        
        // ---------- Enter 键：自动缩进 + Markdown 续行 ----------
        if (e.key === 'Enter' && !e.ctrlKey && !e.metaKey) {
            e.preventDefault();
            var start = ta.selectionStart;
            var val = ta.value;
            var lineStart = val.lastIndexOf('\n', start - 1) + 1;
            var currentLine = val.substring(lineStart, start);
            var indent = currentLine.match(/^(\s*)/)[1];
            
            var trimmed = currentLine.trim();
            var extraIndent = '';
            var prefix = '';
            
            if (trimmed.endsWith('{') || trimmed.endsWith(':')) {
                extraIndent = '    ';
            }
            
            // 无序列表续行 / Unordered list continuation
            var listMatch = trimmed.match(/^([-*+])\s/);
            if (listMatch && trimmed.length > 2) {
                prefix = listMatch[1] + ' ';
            } else if (listMatch && trimmed.length <= 2) {
                // 空列表项，清除 / Empty list item, clear
                ta.value = val.substring(0, lineStart) + val.substring(start);
                ta.selectionStart = ta.selectionEnd = lineStart;
                ta.dispatchEvent(new Event('input', { bubbles: true }));
                return;
            }
            
            // 有序列表续行 / Ordered list continuation
            var olMatch = trimmed.match(/^(\d+)\.\s/);
            if (olMatch && trimmed.length > olMatch[0].length) {
                var nextNum = parseInt(olMatch[1]) + 1;
                prefix = nextNum + '. ';
            } else if (olMatch && trimmed.length <= olMatch[0].length) {
                ta.value = val.substring(0, lineStart) + val.substring(start);
                ta.selectionStart = ta.selectionEnd = lineStart;
                ta.dispatchEvent(new Event('input', { bubbles: true }));
                return;
            }
            
            // 引用续行 / Blockquote continuation
            var bqMatch = trimmed.match(/^>/);
            if (bqMatch && trimmed.length > 1) {
                prefix = '> ';
            }
            
            // 任务列表续行 / Task list continuation
            var taskMatch = trimmed.match(/^[-*+]\s\[[ x]\]\s/);
            if (taskMatch) {
                prefix = '- [ ] ';
            }
            
            var insert = '\n' + indent + extraIndent + prefix;
            ta.value = val.substring(0, start) + insert + val.substring(ta.selectionEnd);
            ta.selectionStart = ta.selectionEnd = start + insert.length;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
            return;
        }
        
        // ---------- 括号自动配对 / Bracket auto-pairing ----------
        var pairs = { '(': ')', '[': ']', '{': '}' };
        var quotes = { '"': '"', "'": "'", '`': '`' };
        
        if (pairs[e.key]) {
            e.preventDefault();
            var start = ta.selectionStart;
            var end = ta.selectionEnd;
            var val = ta.value;
            var selected = val.substring(start, end);
            if (selected.length > 0) {
                ta.value = val.substring(0, start) + e.key + selected + pairs[e.key] + val.substring(end);
                ta.selectionStart = start + 1;
                ta.selectionEnd = end + 1;
            } else {
                ta.value = val.substring(0, start) + e.key + pairs[e.key] + val.substring(end);
                ta.selectionStart = ta.selectionEnd = start + 1;
            }
            ta.dispatchEvent(new Event('input', { bubbles: true }));
            return;
        }
        
        // ---------- 引号自动配对 / Quote auto-pairing ----------
        if (quotes[e.key] && !e.ctrlKey) {
            var start = ta.selectionStart;
            var end = ta.selectionEnd;
            var val = ta.value;
            
            // 光标在配对引号右侧，跳过 / Skip over matching quote
            if (start === end && start < val.length && val[start] === e.key) {
                e.preventDefault();
                ta.selectionStart = ta.selectionEnd = start + 1;
                return;
            }
            
            if (start !== end) {
                e.preventDefault();
                var selected = val.substring(start, end);
                ta.value = val.substring(0, start) + e.key + selected + quotes[e.key] + val.substring(end);
                ta.selectionStart = start + 1;
                ta.selectionEnd = end + 1;
                ta.dispatchEvent(new Event('input', { bubbles: true }));
                return;
            }
            
            e.preventDefault();
            ta.value = val.substring(0, start) + e.key + quotes[e.key] + val.substring(end);
            ta.selectionStart = ta.selectionEnd = start + 1;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
            return;
        }
        
        // ---------- 退格删除配对符号 / Backspace deletes paired symbols ----------
        if (e.key === 'Backspace') {
            var start = ta.selectionStart;
            var end = ta.selectionEnd;
            if (start === end && start > 0) {
                var val = ta.value;
                var before = val[start - 1];
                var after = val[start];
                if ((before === '(' && after === ')') ||
                    (before === '[' && after === ']') ||
                    (before === '{' && after === '}') ||
                    (before === '"' && after === '"') ||
                    (before === "'" && after === "'") ||
                    (before === '`' && after === '`')) {
                    e.preventDefault();
                    ta.value = val.substring(0, start - 1) + val.substring(start + 1);
                    ta.selectionStart = ta.selectionEnd = start - 1;
                    ta.dispatchEvent(new Event('input', { bubbles: true }));
                    return;
                }
            }
        }
    });
    
    // ========== 图片粘贴处理 / Image Paste Handler ==========  
    // 检测剪贴板中的图片，转换为 base64 data URI 插入 Markdown
    // Detect images in clipboard, convert to base64 data URI and insert as Markdown
    ta.addEventListener('paste', function(e) {
        var items = (e.clipboardData || {}).items;
        if (!items) return;
        
        for (var i = 0; i < items.length; i++) {
            if (items[i].type.indexOf('image/') === 0) {
                e.preventDefault();
                var blob = items[i].getAsFile();
                var mime = items[i].type;
                var reader = new FileReader();
                reader.onload = function(evt) {
                    var dataUri = evt.target.result;
                    var start = ta.selectionStart;
                    var end = ta.selectionEnd;
                    var val = ta.value;
                    var timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
                    var alt = 'image_' + timestamp;
                    var md = '![' + alt + '](' + dataUri + ')';
                    ta.value = val.substring(0, start) + md + val.substring(end);
                    ta.selectionStart = ta.selectionEnd = start + md.length;
                    ta.dispatchEvent(new Event('input', { bubbles: true }));
                };
                reader.readAsDataURL(blob);
                return;
            }
        }
    });
    
    // ========== 拖放图片/Markdown 处理 / Drag & Drop Image/Markdown Handler ==========
    // 图片转 base64 插入；Markdown/文本文件内容插入光标处
    // Images → base64 insert; Markdown/text file contents inserted at cursor
    ta.addEventListener('drop', function(e) {
        var files = e.dataTransfer && e.dataTransfer.files;
        if (!files || files.length === 0) return;
        
        for (var i = 0; i < files.length; i++) {
            var file = files[i];
            if (file.type.indexOf('image/') === 0) {
                e.preventDefault();
                e.stopPropagation();
                var reader = new FileReader();
                reader.onload = function(evt) {
                    var dataUri = evt.target.result;
                    var start = ta.selectionStart;
                    var end = ta.selectionEnd;
                    var val = ta.value;
                    var name = file.name.replace(/\.[^.]+$/, '');
                    var md = '![' + name + '](' + dataUri + ')';
                    ta.value = val.substring(0, start) + md + val.substring(end);
                    ta.selectionStart = ta.selectionEnd = start + md.length;
                    ta.dispatchEvent(new Event('input', { bubbles: true }));
                };
                reader.readAsDataURL(file);
                return;
            }
            // Markdown/文本由 Rust ondrop 按路径打开标签；此处不拦截
            // Markdown/text: let Rust ondrop open as tabs by path; do not intercept
        }
    }, true); // capture phase so image drops override parent handler
};

if (window._mm_cmInstance) {
    // 内核已接管跳行与反向同步，勿用 textarea 回退覆盖
    // The kernel already owns jump-to-line and reverse sync; do not overwrite it
} else {
window._mm_scrollToLine = function(lineNumber) {
    var ta = document.querySelector('.editor-textarea');
    if (!ta) return;
    var lines = ta.value.substring(0, ta.value.length).split('\n');
    var pos = 0;
    for (var i = 0; i < Math.min(lineNumber, lines.length); i++) {
        pos += lines[i].length + 1;
    }
    ta.focus();
    ta.setSelectionRange(pos, pos);
    var lineHeight = 22.4;
    ta.scrollTop = lineNumber * lineHeight - ta.clientHeight / 2;
};

window._mm_reverseSyncScroll = (function() {
    var rafId = null;
    var lastRatio = -1;
    return function() {
        if (!window._mm_syncScrollEnabled) return;
        if (rafId !== null) return;
        rafId = requestAnimationFrame(function() {
            rafId = null;
            var preview = document.getElementById('preview-scroll');
            var ta = document.querySelector('.editor-textarea');
            if (!preview || !ta) return;
            var psh = preview.scrollHeight - preview.clientHeight;
            if (psh <= 0) return;
            var ratio = preview.scrollTop / psh;
            if (Math.abs(ratio - lastRatio) < 0.002) return;
            lastRatio = ratio;
            var maxScroll = ta.scrollHeight - ta.clientHeight;
            if (maxScroll > 0) {
                ta.scrollTop = ratio * maxScroll;
            }
        });
    };
})();
}

// 自动初始化 / Auto-initialize
if (document.querySelector('.editor-textarea')) {
    window._mm_initEditor();
}
