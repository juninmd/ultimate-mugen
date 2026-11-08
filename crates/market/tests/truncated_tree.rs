use ume_market::catalog::{detect, parse_tree, Kind};

#[test]
fn truncated_repository_disables_items_needing_tree_followups() {
    let tree = parse_tree(
        r#"{"truncated":true,"tree":[
        {"path":"chars/kfm/kfm.def","type":"blob","size":100},
        {"path":"chars/kfm/kfm.sff","type":"blob","size":5000},
        {"path":"chars/kfm/kfm.air","type":"blob","size":300},
        {"path":"chars/kfm/kfm.cmd","type":"blob","size":200},
        {"path":"chars/kfm/kfm.cns","type":"blob","size":900},
        {"path":"stages/dojo.def","type":"blob","size":60},
        {"path":"stages/dojo.sff","type":"blob","size":7000},
        {"path":"music/theme.ogg","type":"blob","size":100000},
        {"path":"packs/kfm.zip","type":"blob","size":9000000}
    ]}"#,
    )
    .unwrap();
    let catalog = detect(&tree, "", "repo");
    assert!(catalog.truncated);
    assert!(
        !catalog
            .items
            .iter()
            .find(|item| item.kind == Kind::Character)
            .unwrap()
            .installable
    );
    assert!(
        !catalog
            .items
            .iter()
            .find(|item| item.kind == Kind::Stage)
            .unwrap()
            .installable
    );
    assert!(
        catalog
            .items
            .iter()
            .find(|item| item.kind == Kind::Music)
            .unwrap()
            .installable
    );
    assert!(
        catalog
            .items
            .iter()
            .find(|item| item.kind == Kind::Pack)
            .unwrap()
            .installable
    );
}
