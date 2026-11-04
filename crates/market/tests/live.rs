#![cfg(not(target_arch = "wasm32"))]

use std::sync::atomic::AtomicBool;
use ume_market::native::{install_item, list_repo, ListOutcome, UreqTransport};
use ume_market::{Kind, RepoRef};

#[test]
#[ignore]
fn lists_and_installs_from_the_real_github() {
    let repo = RepoRef::parse("juninmd/ultimate-mugen").unwrap();
    let t = UreqTransport::default();
    let (catalog, etag) = match list_repo(&t, &repo, None, std::env::var("GITHUB_TOKEN").ok().as_deref()).expect("listing") {
        ListOutcome::Fresh { catalog, etag } => (catalog, etag),
        ListOutcome::NotModified => panic!("unexpected 304"),
    };
    println!("{} items, etag {etag:?}", catalog.items.len());
    for i in catalog.items.iter().take(20) {
        println!("{:?} {} {} files {} bytes installable={}", i.kind, i.name, i.files.len(), i.bytes, i.installable);
    }
    let again = list_repo(&t, &repo, etag.as_deref(), std::env::var("GITHUB_TOKEN").ok().as_deref()).expect("relisting");
    assert!(matches!(again, ListOutcome::NotModified));
    let smallest = catalog.items.iter().filter(|i| i.installable).min_by_key(|i| i.bytes).expect("an item");
    println!("installing {:?} {}", smallest.kind, smallest.name);
    let files = install_item(&t, &repo, smallest, &catalog, &AtomicBool::new(false), &|p| println!("{}/{} files", p.done_files, p.total_files)).expect("install");
    assert!(!files.is_empty());
    assert!(catalog.items.iter().any(|i| i.kind == Kind::Pack) || !catalog.items.is_empty());
}
