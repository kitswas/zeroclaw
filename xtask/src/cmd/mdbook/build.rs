use crate::cmd::mdbook::refs::{build_api, build_refs};
use crate::util::*;
use std::path::Path;
use std::process::Command;

const DEFAULT_TAG: &str = "master";

pub fn run(tag: Option<&str>) -> anyhow::Result<()> {
    let root = repo_root();
    require_tool("cargo", "https://rustup.rs")?;
    ensure_cargo_tool("mdbook", "mdbook")?;
    ensure_cargo_tool("mdbook-xgettext", "mdbook-i18n-helpers")?;
    ensure_cargo_tool("mdbook-gettext", "mdbook-i18n-helpers")?;
    ensure_cargo_tool("mdbook-mermaid", "mdbook-mermaid")?;

    build_refs(&root)?;
    build_api(&root)?;
    build_locales(&root, tag)?;
    assemble(&root, tag)?;
    println!(
        "==> Done. Open: {}",
        book_dir(&root)
            .join("book")
            .join(tag.unwrap_or(DEFAULT_TAG))
            .join("index.html")
            .display()
    );
    Ok(())
}

pub fn build_locales(root: &std::path::Path, tag: Option<&str>) -> anyhow::Result<()> {
    let book = book_dir(root);
    let entries = locale_entries();
    println!(
        "==> Building mdBook for locales: {}",
        entries
            .iter()
            .map(|e| e.code.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    );
    inject_lang_switcher_locales(&book, &entries)?;
    let mdbook = mdbook_program()?;
    let tag_dir = tag.unwrap_or(DEFAULT_TAG);
    for entry in &entries {
        let dest = format!("book/{}/{}", tag_dir, entry.code);
        run_cmd(
            Command::new(&mdbook)
                .args(["build", "-d", &dest])
                .env("MDBOOK_BOOK__LANGUAGE", &entry.code)
                .current_dir(&book),
        )?;
    }
    Ok(())
}

/// Render `theme/lang-switcher.js.tpl` into `theme/lang-switcher.js` with the
/// `LOCALES` array filled from `locales.toml`. The `.js` output is gitignored
/// (every locale add/remove rewrites it); the `.tpl` source is the tracked
/// truth. Errors loudly when the template is missing — silently skipping
/// would let mdBook fail later with a confusing "missing additional-js"
/// message.
pub fn inject_lang_switcher_locales(book: &Path, entries: &[LocaleEntry]) -> anyhow::Result<()> {
    let tpl_path = book.join("theme/lang-switcher.js.tpl");
    let js_path = book.join("theme/lang-switcher.js");
    let src = std::fs::read_to_string(&tpl_path).map_err(|e| {
        anyhow::Error::msg(format!(
            "lang-switcher.js.tpl missing at {}: {e}. The template is the tracked source of \
             truth for the locale switcher; do not delete it.",
            tpl_path.display(),
        ))
    })?;
    let locale_lines: String = entries
        .iter()
        .map(|e| format!("    {{ code: {:?}, label: {:?} }},", e.code, e.label))
        .collect::<Vec<_>>()
        .join("\n");
    let new_block = format!("const LOCALES = [\n{locale_lines}\n  ];");

    let start = src
        .find("const LOCALES = [")
        .ok_or_else(|| anyhow::Error::msg("lang-switcher.js.tpl: LOCALES array not found"))?;
    let end = src[start..]
        .find("];")
        .ok_or_else(|| anyhow::Error::msg("lang-switcher.js.tpl: LOCALES closing ]; not found"))?;
    let updated = format!("{}{}{}", &src[..start], new_block, &src[start + end + 2..]);
    std::fs::write(&js_path, updated)?;
    Ok(())
}

pub fn print_locales() {
    let codes: Vec<String> = locale_entries().into_iter().map(|e| e.code).collect();
    println!("{}", codes.join(" "));
}

pub fn assemble(root: &std::path::Path, tag: Option<&str>) -> anyhow::Result<()> {
    println!("==> Assembling site (rustdoc + locale redirect)");
    let book = book_dir(root);
    let tag_dir = tag.unwrap_or(DEFAULT_TAG);
    let api_dest = book.join("book").join(tag_dir).join("api");
    let _ = std::fs::remove_dir_all(&api_dest);
    copy_dir_all(root.join("target/doc"), &api_dest)?;

    const INDEX_HTML: &str = "<!doctype html>\n<meta charset=\"utf-8\">\n<meta http-equiv=\"refresh\" content=\"0; url=./en/\">\n<link rel=\"canonical\" href=\"./en/\">\n<title>ZeroClaw Docs</title>\n";
    let out_dir = book.join("book").join(tag_dir);
    std::fs::create_dir_all(&out_dir)?;
    std::fs::write(out_dir.join("index.html"), INDEX_HTML)?;
    // Write small metadata file with the version tag
    let version_meta = format!("{}\n", tag.unwrap_or(DEFAULT_TAG));
    std::fs::write(out_dir.join("_version.txt"), version_meta)?;
    Ok(())
}
