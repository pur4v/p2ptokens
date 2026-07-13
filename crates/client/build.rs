//! Build the embedded web UI (Vite/React widget) before compiling the crate.
//! Produces `webui/dist/` which `rust-embed` bakes into the binary. Requires
//! Node/npm at build time (CI installs it). Set `P2P_SKIP_WEB_BUILD=1` to skip
//! (e.g. when `dist/` is already built) — but a `dist/` must then exist.

use std::path::Path;
use std::process::Command;

fn main() {
    let webui = Path::new(env!("CARGO_MANIFEST_DIR")).join("webui");

    // Rebuild only when the web UI inputs change.
    for p in [
        "src",
        "package.json",
        "package-lock.json",
        "vite.config.ts",
        "tailwind.config.ts",
        "postcss.config.js",
        "tsconfig.json",
        "shell.html",
    ] {
        println!("cargo:rerun-if-changed={}", webui.join(p).display());
    }
    println!("cargo:rerun-if-env-changed=P2P_SKIP_WEB_BUILD");

    let dist = webui.join("dist");
    if std::env::var("P2P_SKIP_WEB_BUILD").is_ok() {
        eprintln!("P2P_SKIP_WEB_BUILD set — skipping web UI build");
        ensure_dist_placeholder(&dist);
        return;
    }

    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };

    // Install deps once (node_modules absent), then build.
    if !webui.join("node_modules").exists() {
        run(&webui, npm, &["install", "--no-audit", "--no-fund"]);
    }
    run(&webui, npm, &["run", "build"]);

    // Vite lib mode emits the JS as chat.min.js and the CSS under some name;
    // normalize the CSS to chat.min.css and drop the shell in as index.html.
    if let Ok(entries) = std::fs::read_dir(&dist) {
        for e in entries.flatten() {
            let p = e.path();
            let is_css = p.extension().map(|x| x == "css").unwrap_or(false);
            let named = p.file_name().and_then(|n| n.to_str()) == Some("chat.min.css");
            if is_css && !named {
                let _ = std::fs::rename(&p, dist.join("chat.min.css"));
            }
        }
    }
    std::fs::copy(webui.join("shell.html"), dist.join("index.html"))
        .expect("copy shell.html -> dist/index.html");
}

/// Guarantee `dist/` exists so the `rust-embed` folder path is valid even when
/// the JS build is skipped.
fn ensure_dist_placeholder(dist: &Path) {
    if !dist.join("index.html").exists() {
        let _ = std::fs::create_dir_all(dist);
        let _ = std::fs::write(
            dist.join("index.html"),
            "<!doctype html><title>p2ptokens</title><p>web UI not built (P2P_SKIP_WEB_BUILD)</p>",
        );
    }
}

fn run(dir: &Path, cmd: &str, args: &[&str]) {
    let status = Command::new(cmd)
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap_or_else(|e| panic!("failed to run `{cmd} {}`: {e}", args.join(" ")));
    if !status.success() {
        panic!("`{cmd} {}` failed with {status}", args.join(" "));
    }
}
