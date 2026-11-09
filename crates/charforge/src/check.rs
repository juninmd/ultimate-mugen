use std::path::Path;
use std::sync::Arc;
use ume_engine::loader::load_mugen_char;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

pub struct CheckReport {
    pub lines: Vec<String>,
    pub unsupported: usize,
    pub failed: usize,
}

pub fn check_directory(dir: &Path) -> CheckReport {
    let mut report = CheckReport { lines: Vec::new(), unsupported: 0, failed: 0 };
    let mut vfs = Vfs::new();
    if let Err(error) = add_dir(&mut vfs, dir, "c") {
        report.lines.push(format!("{}: io error {error}", dir.display()));
        report.failed += 1;
        return report;
    }
    for def in find_char_defs(&vfs) {
        match load_mugen_char(&vfs, &def, 1, true) {
            Ok(character) => {
                let character = Arc::new(character);
                let diagnostics = character.diagnostics();
                if diagnostics.is_clean() {
                    report.lines.push(format!("DIAG {def}: clean"));
                } else {
                    report.unsupported += 1;
                    report.lines.push(format!("DIAG {def}: {}", diagnostics.summary()));
                    report.lines.push(diagnostics.detail());
                }
                report.lines.push(format!("OK {def} | anims={} states={} sprites={}", character.anims.len(), character.states.len(), character.sprites.len()));
            }
            Err(error) => {
                report.failed += 1;
                report.lines.push(format!("FAIL load {def}: {error}"));
            }
        }
    }
    report
}
