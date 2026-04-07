                                                                                                                                           /**
 * MarkdownMonkey 编辑器增强脚本
 * Editor enhancement script
 * 
 * 功能/Features:
 * - 搜索匹配高亮 / Search match highlighting
 * - Tab 键缩进/反缩进 / Tab indent/outdent
 * - Enter 自动缩进 + Markdown 列表续行 / Enter auto-indent + Markdown list continuation
 * - 括号/引号自动配对 / Bracket/quote auto-pairing
 * - 退格删除配对符号 / Backspace deletes paired symbols
 */
// 使用全局初始化函数，支持 Dioxus 重新创建 textarea 时重新附加增强功能
// Use global init function to support re-attaching when Dioxus recreates the textarea
window._mm_initEditor = function() {
    var ta = document.querySelector('.editor-textarea');
    if (!ta) return;
    // 如果已有增强且 textarea 未被替换，跳过
    // If already enhanced and textarea hasn't been replaced, skip
    if (ta._mm_enhanced) return;
    ta._mm_enhanced = true;
    
    // ========== 搜索高亮层 / Search Highlight Overlay ==========
    var highlightDiv = document.createElement('div');
    highlightDiv.className = 'editor-highlight-overlay';
    highlightDiv.id = 'editor-highlight-overlay';
    // 必须设置 position:absolute 防止占据正常流空间导致编辑区偏移
    // Must set position:absolute to prevent taking flow space and shifting editor
    highlightDiv.style.position = 'absolute';
    highlightDiv.style.pointerEvents = 'none';
    highlightDiv.style.zIndex = '0';
    ta.parentNode.insertBefore(highlightDiv, ta);
    // 确保 textarea 在高亮层之上 / Ensure textarea is above highlight overlay
    ta.style.position = 'relative';
    ta.style.zIndex = '1';
    
    function syncHighlight() {
        highlightDiv.style.width = ta.clientWidth + 'px';
        highlightDiv.style.height = ta.clientHeight + 'px';
        highlightDiv.style.padding = getComputedStyle(ta).padding;
        highlightDiv.style.font = getComputedStyle(ta).font;
        highlightDiv.style.lineHeight = getComputedStyle(ta).lineHeight;
        highlightDiv.style.letterSpacing = getComputedStyle(ta).letterSpacing;
        highlightDiv.style.whiteSpace = 'pre-wrap';
        highlightDiv.style.wordWrap = 'break-word';
        highlightDiv.style.overflow = 'hidden';
        highlightDiv.style.top = ta.offsetTop + 'px';
        highlightDiv.style.left = ta.offsetLeft + 'px';
    }
    syncHighlight();
    
    // 滚动同步 / Scroll sync
    ta.addEventListener('scroll', function() {
        highlightDiv.scrollTop = ta.scrollTop;
        highlightDiv.scrollLeft = ta.scrollLeft;
    });

    // 同步滚动：在 JS 侧直接处理，避免经过 Dioxus 信号路由造成性能开销
    // Sync scroll: handled directly in JS to avoid Dioxus signal routing overhead
    var syncScrollEnabled = true;

    window._mm_setSyncScroll = function(enabled) {
        syncScrollEnabled = enabled;
    };

    var rafId = null;
    var lastRatio = -1;
    ta.addEventListener('scroll', function() {
        if (!syncScrollEnabled) return;
        // rAF 节流，最多每帧一次(约16ms)
        // rAF throttle, at most once per frame (~16ms)
        if (rafId !== null) return;
        rafId = requestAnimationFrame(function() {
            rafId = null;
            var sh = ta.scrollHeight - ta.clientHeight;
            if (sh <= 0) return;
            var ratio = ta.scrollTop / sh;
            // 只有比例变化超过阈值才更新，减少 DOM 操作
            // Only update when ratio changes beyond threshold to reduce DOM operations
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
    
    // 全局搜索高亮函数 / Global search highlight function
    window._mm_highlightSearch = function(query, caseInsensitive, currentIndex) {
        if (!query) {
            highlightDiv.innerHTML = '';
            return;
        }
        var content = ta.value;
        var escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        var flags = caseInsensitive ? 'gi' : 'g';
        var regex = new RegExp(escaped, flags);
        var result = '';
        var lastIndex = 0;
        var matchIndex = 0;
        var match;
        while ((match = regex.exec(content)) !== null) {
            if (match.index > lastIndex) {
                result += escapeHtml(content.substring(lastIndex, match.index));
            }
            var cls = (matchIndex === currentIndex) ? 'search-highlight-current' : 'search-highlight';
            result += '<mark class="' + cls + '">' + escapeHtml(match[0]) + '</mark>';
            lastIndex = regex.lastIndex;
            matchIndex++;
            if (matchIndex > 2000) break; // 安全限制 / Safety limit
        }
        if (lastIndex < content.length) {
            result += escapeHtml(content.substring(lastIndex));
        }
        result += '\n'; // 末尾换行确保高度一致 / Trailing newline for height consistency
        highlightDiv.innerHTML = result;
        syncHighlight();
    };
    
    function escapeHtml(s) {
        return s.replace(/\x26/g, '\x26amp;').replace(/\x3C/g, '\x26lt;').replace(/\x3E/g, '\x26gt;');
    }
    
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
    
    // ========== 拖放图片处理 / Drag & Drop Image Handler ==========
    // 检测拖放的图片文件，转换为 base64 插入 Markdown
    // Detect dropped image files, convert to base64 and insert as Markdown
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
        }
    }, true); // capture phase to override parent handler
};

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

window._mm_reverseSyncScroll = function() {
    var preview = document.getElementById('preview-scroll');
    var ta = document.querySelector('.editor-textarea');
    if (!preview || !ta) return;
    var psh = preview.scrollHeight - preview.clientHeight;
    if (psh <= 0) return;
    var ratio = preview.scrollTop / psh;
    var maxScroll = ta.scrollHeight - ta.clientHeight;
    if (maxScroll > 0) {
        ta.scrollTop = ratio * maxScroll;
    }
};

// 自动初始化 / Auto-initialize
if (document.querySelector('.editor-textarea')) {
    window._mm_initEditor();
}
