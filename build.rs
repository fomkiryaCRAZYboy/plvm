use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let pli_src = out_dir.join("PLI");
    let pli_build = pli_src.join("build");
    let pli_include = pli_src.join("include");

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

    /* generate Rust bindings from C parser.h */
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", pli_include.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("c_ast_binds.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rustc-link-search=native={}", pli_build.join("lib").display());
    println!("cargo:rustc-link-lib=static=pli");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
}
