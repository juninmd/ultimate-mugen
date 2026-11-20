use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut spriteSet = HashMap::new();
    let mut spriteAnimations = HashMap::new();
    if let Some(sprite_set_path) = args.get(1) {
        let sprite_set_path = sprite_set_path.unwrap();
        let sprite_set_path = PathBuf::from(sprite_set_path);
        if !sprite_set_path.exists() {
            eprintln!("Error: sprite_set file does not exist");
            process::exit(1);
        }
        let sprite_set_content = std::fs::read_to_string(&sprite_set_path).expect("Failed to read sprite_set");
        if sprite_set_content.trim().is_empty() {
            eprintln!("Error: sprite_set is empty");
            process::exit(1);
        }
    }

    // ... rest of the build process
}