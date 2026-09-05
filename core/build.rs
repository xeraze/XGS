use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let scanner_dir = manifest_dir.parent().unwrap().join("scanner");

    let dst = cmake::Config::new(scanner_dir.to_str().unwrap())
        .generator("MinGW Makefiles")
        .build();

    let lib_path = dst.join("lib");
    let bin_path = dst.join("bin");

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-search=native={}", bin_path.display());
    println!("cargo:rustc-link-lib=dylib=scanner");

    println!("cargo:rerun-if-changed=../scanner/");
    println!("cargo:rerun-if-changed=src/");
}
