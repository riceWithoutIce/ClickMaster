fn main() {
    let version = std::fs::read_to_string("../version/VERSION").expect("VERSION required");
    assert_eq!(
        version.trim(),
        std::env::var("CARGO_PKG_VERSION").unwrap(),
        "Run scripts/version.mjs --sync"
    );
    println!("cargo:rerun-if-changed=../version/VERSION");
    // Windows application and default window icons come from the compiled PE resource.
    // Tauri's other rerun rules must not leave an updated icon outside Cargo tracking.
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=icons/icon.png");
    tauri_build::build()
}
