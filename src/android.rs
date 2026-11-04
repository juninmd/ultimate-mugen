#![cfg(target_os = "android")]

include!("main.rs");

#[no_mangle]
fn android_main(app: ume_shell::AndroidApp) {
    let _ = ume_shell::ANDROID_APP.set(app);
    main();
}
