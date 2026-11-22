use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-env-changed=UME_SPRITE_PACKS");
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("embedded_sprites.rs");
    let mut body = String::from("pub static EMBEDDED: &[(&str, &[u8])] = &[\n");
    if let Ok(dir) = env::var("UME_SPRITE_PACKS") {
        println!("cargo:rerun-if-changed={dir}");
        let mut packs: Vec<_> = fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("UME_SPRITE_PACKS {dir}: {e}"))
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "pack") && !p.to_string_lossy().ends_with(".menu.pack"))
            .collect();
        packs.sort();
        for p in packs {
            let id = p.file_stem().unwrap().to_string_lossy().to_string();
            let abs = fs::canonicalize(&p).unwrap();
            body.push_str(&format!("    ({id:?}, include_bytes!({:?})),\n", abs.to_string_lossy()));
        }
    }
    body.push_str("];\n");
    fs::write(out, body).unwrap();
}
