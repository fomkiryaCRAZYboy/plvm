use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let pli_src = out_dir.join("PLI");
    let pli_build = pli_src.join("build");

    if !pli_src.exists() {
        let status = Command::new("git")
            .args([
                "clone",
                "--branch", "libpli",
                "--depth", "1",
                "https://github.com/fomkiryaCRAZYboy/PLI.git",
            ])
            .arg(&pli_src)
            .status()
            .expect("failed to run git clone");
        assert!(status.success(), "git clone failed");
    }

    std::fs::create_dir_all(&pli_build).unwrap();

    let status = Command::new("cmake")
        .arg("..")
        .current_dir(&pli_build)
        .status()
        .expect("failed to run cmake configure");
    assert!(status.success(), "cmake configure failed");

    let status = Command::new("cmake")
        .args(["--build", ".", "--target", "pli", "--parallel"])
        .current_dir(&pli_build)
        .status()
        .expect("failed to run cmake build");
    assert!(status.success(), "cmake build failed");

    println!("cargo:rustc-link-search=native={}", pli_build.join("lib").display());
    println!("cargo:rustc-link-lib=static=pli");
    println!("cargo:rerun-if-changed=build.rs");
}
