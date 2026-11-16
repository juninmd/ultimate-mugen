use std::path::{Path, PathBuf};
use ume_formats::health;
use ume_formats::ini::{find_section, parse_ini_no_lines};
use ume_formats::vfs::{add_dir, add_zip, text, Vfs};

#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug)]
pub struct Finding {
    pub source: String,
    pub def: String,
    pub verdict: Verdict,
    pub problems: Vec<String>,
    pub warnings: Vec<String>,
}

fn has_files_section(vfs: &Vfs, def: &str) -> bool {
    vfs.get(def)
        .map(|file| find_section(&parse_ini_no_lines(&text(file.as_slice())), "files").is_some())
        .unwrap_or(false)
}

pub fn check_vfs(source: &str, vfs: &Vfs) -> Vec<Finding> {
    check_vfs_with(source, vfs, false)
}

pub fn check_vfs_with(source: &str, vfs: &Vfs, run: bool) -> Vec<Finding> {
    let mut defs: Vec<String> = vfs
        .keys()
        .filter(|key| key.to_ascii_lowercase().ends_with(".def"))
        .filter(|key| has_files_section(vfs, key))
        .cloned()
        .collect();
    defs.sort();
    defs.into_iter()
        .map(|def| {
            let mut problems = health::problems(vfs, &def);
            if run && problems.is_empty() {
                problems = crate::homologate_run::runtime_problems(vfs, &def);
            }
            let warnings = health::warnings(vfs, &def);
            let verdict = match (problems.is_empty(), warnings.is_empty()) {
                (false, _) => Verdict::Fail,
                (true, false) => Verdict::Warn,
                (true, true) => Verdict::Pass,
            };
            Finding {
                source: source.to_string(),
                def,
                verdict,
                problems,
                warnings,
            }
        })
        .collect()
}

pub fn check_path(path: &Path, run: bool) -> Result<Vec<Finding>, String> {
    let source = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut vfs = Vfs::new();
    if path.is_dir() {
        add_dir(&mut vfs, path, "").map_err(|error| format!("{}: {error}", path.display()))?;
    } else {
        let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        add_zip(&mut vfs, &bytes, "").map_err(|error| format!("{}: {error:?}", path.display()))?;
    }
    Ok(check_vfs_with(&source, &vfs, run))
}

pub fn report(findings: &[Finding]) -> String {
    let count = |verdict: Verdict| findings.iter().filter(|f| f.verdict == verdict).count();
    let mut out = format!(
        "# Homologação MUGEN\n\nTotal: {} · PASS: {} · WARN: {} · FAIL: {}\n\n",
        findings.len(),
        count(Verdict::Pass),
        count(Verdict::Warn),
        count(Verdict::Fail)
    );
    for finding in findings {
        let tag = match finding.verdict {
            Verdict::Pass => "PASS",
            Verdict::Warn => "WARN",
            Verdict::Fail => "FAIL",
        };
        out.push_str(&format!(
            "- **{tag}** `{}` · {}\n",
            finding.source, finding.def
        ));
        for line in finding.problems.iter().chain(finding.warnings.iter()) {
            out.push_str(&format!("  - {line}\n"));
        }
    }
    out
}

pub fn collect_inputs(root: &Path) -> Vec<PathBuf> {
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(root)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .collect()
        })
        .unwrap_or_default();
    inputs.retain(|p| p.is_dir() || p.extension().is_some_and(|x| x.eq_ignore_ascii_case("zip")));
    inputs.sort();
    inputs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(def: &str, files: &[&str]) -> Vfs {
        let mut vfs = Vfs::new();
        vfs.insert("c/fighter.def".into(), def.as_bytes().to_vec());
        for name in files {
            vfs.insert(format!("c/{name}"), b"x".to_vec());
        }
        vfs
    }

    #[test]
    fn grades_complete_warn_and_fail_characters() {
        let full = "[Files]\nsprite = f.sff\nanim = f.air\ncns = f.cns\ncmd = f.cmd\n";
        let warn = "[Files]\nsprite = f.sff\nanim = f.air\ncns = f.cns\ncmd = f.cmd\n";
        let fail = "[Files]\nsprite = f.sff\nanim = f.air\n";
        let verdicts: Vec<Verdict> = [
            pack(full, &["f.sff", "f.air", "f.cns", "f.cmd"]),
            pack(warn, &["f.sff", "f.air", "f.cns"]),
            pack(fail, &["f.air"]),
        ]
        .iter()
        .map(|vfs| check_vfs("t", vfs).remove(0).verdict)
        .collect();
        assert_eq!(verdicts, vec![Verdict::Pass, Verdict::Warn, Verdict::Fail]);
    }

    #[test]
    fn ignores_stage_definitions_without_files_section() {
        let vfs = pack("[Info]\nname = stage\n", &[]);
        assert!(check_vfs("t", &vfs).is_empty());
    }
}
