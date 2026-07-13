// Build script: compile the tree-sitter-pike C parser sources into
// a static library and emit cargo:rustc-link-lib=static=pike_grammar.
//
// Source commit is pinned to match the extension's grammar pin
// (a8bc7e3d6064f67e05b35cd9d8406c7a41059131, tree-sitter-pike v1.3.1).
// When the upstream commit moves, update the SHA in `extension.toml`,
// the `tree-sitter-pike` rev in the workspace `Cargo.toml`, and here.

use std::path::PathBuf;

const GRAMMAR_COMMIT: &str = "a8bc7e3d6064f67e05b35cd9d8406c7a41059131";

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR set by cargo"));
    let grammar_dir = out_dir.join(format!("tree-sitter-pike-{GRAMMAR_COMMIT}"));

    // First-time setup: fetch the pinned commit of the grammar repo
    // into OUT_DIR. Subsequent builds reuse the cached checkout.
    if !grammar_dir.join("src/parser.c").is_file() {
        std::fs::create_dir_all(&grammar_dir).expect("mkdir grammar_dir");
        let url =
            format!("https://github.com/TheSmuks/tree-sitter-pike/archive/{GRAMMAR_COMMIT}.tar.gz");
        let archive = out_dir.join("grammar.tar.gz");
        download(&url, &archive);
        extract(&archive, &out_dir);
        // The tarball unpacks to a directory named
        // `tree-sitter-pike-<short_sha>/`. Find it and rename.
        let entries = std::fs::read_dir(&out_dir)
            .expect("read_dir OUT_DIR")
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("tree-sitter-pike-")
            })
            .collect::<Vec<_>>();
        for entry in entries {
            let from = entry.path();
            if from != grammar_dir {
                std::fs::rename(&from, &grammar_dir).expect("rename grammar checkout");
                break;
            }
        }
        // Sanity check.
        assert!(
            grammar_dir.join("src/parser.c").is_file(),
            "tree-sitter-pike checkout at {} is missing src/parser.c",
            grammar_dir.display()
        );
    }

    let src = grammar_dir.join("src");
    let mut build = cc::Build::new();
    build
        .file(src.join("parser.c"))
        .file(src.join("scanner.c"))
        .include(&src)
        .include(src.join("tree_sitter"))
        .opt_level(2)
        .flag_if_supported("-w"); // tree-sitter parsers are noisy by default

    // The grammar's C sources are POSIX-flavoured. On Windows we
    // skip the build (the bridge path is non-Windows in this
    // change; the SSH transport can be added later if needed).
    if cfg!(target_os = "windows") {
        println!("cargo:warning=pike-lsp: skipping tree-sitter-pike build on Windows");
        return;
    }

    build.compile("pike_grammar");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PIKE_LSP_GRAMMAR_COMMIT");
}

fn download(url: &str, dest: &PathBuf) {
    let status = std::process::Command::new("curl")
        .args(["-fsSL", "--retry", "3", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .expect("curl available");
    assert!(status.success(), "download of {url} failed");
}

fn extract(archive: &PathBuf, out: &PathBuf) {
    let status = std::process::Command::new("tar")
        .args(["-xzf"])
        .arg(archive)
        .arg("-C")
        .arg(out)
        .status()
        .expect("tar available");
    assert!(status.success(), "extract of {} failed", archive.display());
}
