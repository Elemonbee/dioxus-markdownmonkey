/**
 * 把完整 HTML 放进隐藏 iframe 并打开系统打印框（可另存 PDF）
 * Load a full HTML document into a hidden iframe and open the system print dialog
 */
window._mm_printHtml = function (html) {
    if (typeof html !== 'string' || !html) return;
    var old = document.getElementById('mm-print-frame');
    if (old) old.remove();
    var iframe = document.createElement('iframe');
    iframe.id = 'mm-print-frame';
    iframe.setAttribute('aria-hidden', 'true');
    iframe.style.cssText = 'position:fixed;right:0;bottom:0;width:0;height:0;border:0;visibility:hidden;';
    document.body.appendChild(iframe);
    var win = iframe.contentWindow;
    var doc = iframe.contentDocument;
    if (!win || !doc) return;
    doc.open();
    doc.write(html);
    doc.close();
    var printed = false;
    var printOnce = function () {
        if (printed) return;
        printed = true;
        try {
            win.focus();
            win.print();
        } catch (err) { /* ignore */ }
        setTimeout(function () {
            if (iframe.parentNode) iframe.remove();
        }, 1500);
    };
    var tries = 0;
    var waitReady = function () {
        tries += 1;
        var pendingImg = 0;
        var imgs = doc.images || [];
        for (var i = 0; i < imgs.length; i++) {
            if (!imgs[i].complete) pendingImg += 1;
        }
        var mermaidNodes = doc.querySelectorAll('pre.mermaid');
        var mermaidLeft = 0;
        for (var j = 0; j < mermaidNodes.length; j++) {
            if (!mermaidNodes[j].querySelector('svg')) mermaidLeft += 1;
        }
        if (tries > 50 || (pendingImg === 0 && mermaidLeft === 0)) {
            setTimeout(printOnce, 120);
            return;
        }
        setTimeout(waitReady, 100);
    };
    if (doc.readyState === 'complete') {
        setTimeout(waitReady, 80);
    } else {
        iframe.onload = function () { setTimeout(waitReady, 80); };
    }
};
