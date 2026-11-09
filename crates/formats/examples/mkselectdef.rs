use std::fs;
use std::path::{Path, PathBuf};
use ume_formats::audit::select_def::build_select;
use ume_formats::ini::{find_section, parse_ini_no_lines};

fn collect_defs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_defs(&path, out);
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("def"))
        {
            out.push(path);
        }
    }
}

fn relative(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn is_character(text: &str) -> bool {
    let sections = parse_ini_no_lines(text);
    find_section(&sections, "files").is_some_and(|f| f.get("cns").is_some())
}

fn is_stage(text: &str) -> bool {
    let sections = parse_ini_no_lines(text);
    find_section(&sections, "stageinfo").is_some()
}

fn scan(root: &Path, folder: &str, keep: fn(&str) -> bool) -> Vec<String> {
    let mut defs = Vec::new();
    collect_defs(&root.join(folder), &mut defs);
    defs.into_iter()
        .filter(|p| {
            fs::read(p)
                .map(|b| keep(&String::from_utf8_lossy(&b)))
                .unwrap_or(false)
        })
        .map(|p| relative(&root.join(folder), &p))
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(root) = args.first().filter(|a| !a.starts_with("--")) else {
        eprintln!("usage: mkselectdef <mugen_root> [--template <select.def>]");
        std::process::exit(2);
    };
    let root = PathBuf::from(root);
    let template = args
        .iter()
        .position(|a| a == "--template")
        .and_then(|i| args.get(i + 1));
    let template =
        template.map(|p| fs::read_to_string(p).unwrap_or_else(|e| panic!("template {p}: {e}")));
    let characters = scan(&root, "chars", is_character);
    let stages: Vec<String> = scan(&root, "stages", is_stage)
        .into_iter()
        .map(|s| format!("stages/{s}"))
        .collect();
    print!(
        "{}",
        build_select(&characters, &stages, template.as_deref())
    );
}
