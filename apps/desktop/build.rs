fn main() {
    println!("cargo:rerun-if-changed=beni.rc");
    println!("cargo:rerun-if-changed=../../assets/app-icon/Beni.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("beni.rc", embed_resource::NONE)
            .manifest_required()
            .expect("Beni's Windows icon resource must compile");
    }
}
