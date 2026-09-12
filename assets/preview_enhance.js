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
    window._mm_enhancePreviewRoot = function (root, dark) {
        if (!root) return;
        renderMath(root);
        renderMermaid(root, dark === true);
    };

    /**
     * 渲染当前预览窗格 / Enhance the live preview pane
     */
    window._mm_enhancePreview = function () {
        var root = document.querySelector('.preview-html-root');
        if (!root) return;
        var needMath = !!root.querySelector('.math-inline, .math-display');
        var needMermaid = !!root.querySelector('pre.mermaid');
        if ((needMath && !window.katex) || (needMermaid && !window.mermaid)) {
            if (!window._mm_previewEnhanceTries) window._mm_previewEnhanceTries = 0;
            if (window._mm_previewEnhanceTries < 50) {
                window._mm_previewEnhanceTries += 1;
                setTimeout(window._mm_enhancePreview, 80);
                return;
            }
        }
        window._mm_previewEnhanceTries = 0;
        window._mm_enhancePreviewRoot(root, isDarkTheme());
    };
})();
