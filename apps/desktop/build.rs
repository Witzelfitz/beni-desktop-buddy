fn main() {
    println!("cargo:rerun-if-changed=beni.rc");
    println!("cargo:rerun-if-changed=../../assets/app-icon/Beni.ico");
    println!("cargo:rerun-if-changed=src/updater/macos.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/updater/macos.m")
            .flag("-fobjc-arc")
            .flag("-fblocks")
            .flag("-mmacosx-version-min=11.0")
            .compile("beni_updater");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("beni.rc", embed_resource::NONE)
            .manifest_required()
            .expect("Beni's Windows icon resource must compile");
    }
}
