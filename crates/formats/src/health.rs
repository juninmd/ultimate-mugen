use crate::ini::{find_section, parse_ini_no_lines, unquote_str};
use crate::vfs::{norm_path, text, Vfs};

const REQUIRED: [&str; 3] = ["sprite", "anim", "cns"];

pub fn problems(vfs: &Vfs, def_path: &str) -> Vec<String> {
    let def_path = norm_path(def_path);
    let base = def_path.rsplit_once('/').map(|(b, _)| b.to_string()).unwrap_or_default();
    let Some(raw) = vfs.get(&def_path) else {
        return vec!["arquivo .def não encontrado".into()];
    };
    let secs = parse_ini_no_lines(&text(raw));
    let Some(files) = find_section(&secs, "files") else {
        return vec!["sem seção [Files]".into()];
    };
    REQUIRED
        .iter()
        .filter_map(|key| match files.get(key).map(unquote_str).filter(|v| !v.is_empty()) {
            None => Some(format!("[Files] sem '{key}'")),
            Some(name) if !in_pack(vfs, &base, name) => Some(format!("{key} '{name}' não existe")),
            Some(_) => None,
        })
        .collect()
}

fn in_pack(vfs: &Vfs, base: &str, name: &str) -> bool {
    let n = norm_path(&unquote_str(name).replace('\\', "/"));
    let base = norm_path(base.trim_end_matches(['/', '\\']));
    let own = if base.is_empty() { n.clone() } else { format!("{base}/{n}") };
    [own, n.clone(), format!("data/{n}")].iter().any(|k| vfs.contains_key(k))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(def: &str, files: &[(&str, &str)]) -> Vfs {
        let mut vfs = Vfs::new();
        vfs.insert("c/fighter.def".into(), def.as_bytes().to_vec());
        for (name, body) in files {
            vfs.insert(format!("c/{name}"), body.as_bytes().to_vec());
        }
        vfs
    }

    const DEF: &str = "[Files]\nsprite = f.sff\nanim = f.air\ncns = f.cns\n";

    #[test]
    fn complete_character_has_no_problems() {
        let vfs = pack(DEF, &[("f.sff", "x"), ("f.air", "x"), ("f.cns", "x")]);
        assert!(problems(&vfs, "c/fighter.def").is_empty());
    }

    #[test]
    fn missing_sprite_file_is_reported() {
        let vfs = pack(DEF, &[("f.air", "x"), ("f.cns", "x")]);
        assert_eq!(problems(&vfs, "c/fighter.def"), vec!["sprite 'f.sff' não existe"]);
    }

    #[test]
    fn missing_sections_and_keys_are_reported() {
        let vfs = pack("[Info]\nname = x\n", &[]);
        assert_eq!(problems(&vfs, "c/fighter.def"), vec!["sem seção [Files]"]);
        let vfs = pack("[Files]\nsprite = f.sff\n", &[("f.sff", "x")]);
        assert_eq!(problems(&vfs, "c/fighter.def").len(), 2);
    }

    #[test]
    fn file_borrowed_from_another_folder_is_not_accepted() {
        let mut vfs = pack(DEF, &[("f.air", "x"), ("f.cns", "x")]);
        vfs.insert("c/other/f.sff".into(), b"x".to_vec());
        assert_eq!(problems(&vfs, "c/fighter.def"), vec!["sprite 'f.sff' não existe"]);
    }

    #[test]
    fn missing_def_is_reported() {
        assert_eq!(problems(&Vfs::new(), "c/fighter.def"), vec!["arquivo .def não encontrado"]);
    }
}
