/**
 * 把 CodeMirror 6 ESM 打成桌面 WebView 可 eval 的 IIFE
 * Bundle CodeMirror 6 ESM into an IIFE the desktop WebView can eval
 */
import * as esbuild from "esbuild";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const outfile = resolve(here, "../../assets/vendor/codemirror6.bundle.js");

await esbuild.build({
    entryPoints: [resolve(here, "src/index.js")],
    bundle: true,
    minify: true,
    format: "iife",
    platform: "browser",
    target: ["es2020"],
    outfile,
    logLevel: "info",
});

console.log(`wrote ${outfile}`);
