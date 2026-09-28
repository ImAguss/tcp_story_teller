fn main() {
    #[cfg(target_os = "windows")]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let lib_dir = std::path::PathBuf::from(manifest_dir).join("lib");
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    }
}

