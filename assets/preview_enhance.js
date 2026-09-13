/**
 * MarkdownMonkey 预览增强：KaTeX 公式 + Mermaid 图表
 * Preview enhance: KaTeX math + Mermaid diagrams
 */
(function () {
    if (window._mm_previewEnhanceInstalled) return;
    window._mm_previewEnhanceInstalled = true;

    function isDarkTheme() {
        var root = document.querySelector('.app-container') || document.documentElement;
        return (root.getAttribute('data-theme') || 'dark') !== 'light';
    }

    function renderMath(root) {
        if (!window.katex) return;
        var nodes = root.querySelectorAll('.math-inline, .math-display');
        for (var i = 0; i < nodes.length; i++) {
            var el = nodes[i];
            if (el.getAttribute('data-mm-math') === '1') continue;
            var tex = el.textContent || '';
            if (!tex.trim()) continue;
            try {
                window.katex.render(tex, el, {
                    throwOnError: false,
                    displayMode: el.classList.contains('math-display'),
                    output: 'html',
                    trust: false,
                    strict: 'ignore'
                });
                el.setAttribute('data-mm-math', '1');
            } catch (err) {
                el.setAttribute('data-mm-math', 'err');
            }
        }
    }

    function renderMermaid(root, dark) {
        if (!window.mermaid) return;
        var nodes = root.querySelectorAll('pre.mermaid');
        if (!nodes.length) return;
        try {
            window.mermaid.initialize({
                startOnLoad: false,
                securityLevel: 'strict',
                theme: dark ? 'dark' : 'default'
            });
        } catch (err) {
            return;
        }
        var pending = [];
        for (var i = 0; i < nodes.length; i++) {
            if (!nodes[i].getAttribute('data-processed')) {
                pending.push(nodes[i]);
            }
        }
        if (!pending.length) return;
        try {
            window.mermaid.run({
                nodes: pending,
                suppressErrors: true
            });
        } catch (err) {
            /* 保留源码 / Keep source when rendering fails */
        }
    }

    /**
     * 渲染指定根节点内的公式与图表
     * Render math and diagrams inside a root element
     */
    /**
     * 预览点击块级元素时跳到源码对应行
     * Jump to the matching source line when a preview block is clicked
     */
    function bindPreviewJump(root) {
        if (!root || root._mm_jumpBound) return;
        root._mm_jumpBound = true;
        root.addEventListener('click', function (event) {
            if (event.target.closest('a, button, input, textarea, .mermaid')) return;
            var el = event.target.closest('[data-source-line]');
            if (!el || !root.contains(el)) return;
            var line = parseInt(el.getAttribute('data-source-line'), 10);
            if (isNaN(line) || !window._mm_scrollToLine) return;
            window._mm_scrollToLine(line);
        });
    }

    window._mm_enhancePreviewRoot = function (root, dark) {
        if (!root) return;
        bindPreviewJump(root);
        renderMath(root);
        renderMermaid(root, dark === true);
    };

    /**
     * 渲染当前预览窗格 / Enhance the live preview pane
     */
    /**
     * 按需插入 Mermaid 脚本，避免启动时解析 2.5MB
     * Load Mermaid on demand so startup does not parse 2.5MB
     */
    function ensureMermaid(done) {
        if (window.mermaid) {
            done();
            return;
        }
        var src = window._mm_mermaidSrc;
        if (!src) {
            done();
            return;
        }
        if (window._mm_mermaidWaiters) {
            window._mm_mermaidWaiters.push(done);
            return;
        }
        window._mm_mermaidWaiters = [done];
        var script = document.createElement('script');
        script.src = src;
        script.onload = function () {
            var waiters = window._mm_mermaidWaiters || [];
            window._mm_mermaidWaiters = null;
            for (var i = 0; i < waiters.length; i++) waiters[i]();
        };
        script.onerror = function () {
            window._mm_mermaidWaiters = null;
            done();
        };
        document.head.appendChild(script);
    }

    window._mm_enhancePreview = function () {
        var root = document.querySelector('.preview-html-root');
        if (!root) return;
        bindPreviewJump(root);
        var needMath = !!root.querySelector('.math-inline, .math-display');
        var needMermaid = !!root.querySelector('pre.mermaid');
        if (needMath && !window.katex) {
            if (!window._mm_previewEnhanceTries) window._mm_previewEnhanceTries = 0;
            if (window._mm_previewEnhanceTries < 50) {
                window._mm_previewEnhanceTries += 1;
                setTimeout(window._mm_enhancePreview, 80);
                return;
            }
        }
        window._mm_previewEnhanceTries = 0;
        var finish = function () {
            window._mm_enhancePreviewRoot(root, isDarkTheme());
        };
        if (needMermaid && !window.mermaid) {
            ensureMermaid(finish);
            return;
        }
        finish();
    };
})();
