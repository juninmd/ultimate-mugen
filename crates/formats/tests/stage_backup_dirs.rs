//! Author backup folders hold stale copies of a stage: their `.def` files must not join the roster.
use ume_formats::vfs::{find_stage_defs, Vfs};

const STAGE_DEF: &str = "[BGdef]\nspr = s.sff\n[StageInfo]\nname = \"x\"\n[Camera]\nboundleft = -150\nboundright = 150\n";

fn pack() -> Vfs {
    let mut vfs = Vfs::new();
    for key in ["stages/good.def", "stages/backup/good.def", "stages/Backup/copy.def"] {
        vfs.insert(key.to_string(), STAGE_DEF.as_bytes().to_vec());
    }
    vfs
}

#[test]
fn stage_backup_folders_do_not_join_the_roster() {
    assert_eq!(find_stage_defs(&pack()), vec!["stages/good.def".to_string()]);
}

#[test]
fn a_folder_whose_name_contains_backup_is_still_a_stage() {
    let mut vfs = pack();
    vfs.insert("stages/backup-temple/temple.def".into(), STAGE_DEF.as_bytes().to_vec());
    assert!(find_stage_defs(&vfs).contains(&"stages/backup-temple/temple.def".to_string()));
}
