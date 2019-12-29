use std::process::Command;
use std::env;
use std::path::Path;
use llvm_tools::{LlvmTools, exe};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let llvm_tools = LlvmTools::new().expect("LLVM tools not found");
    let objcopy = llvm_tools.tool(&exe("llvm-objcopy")).expect("llvm-objcopy not found");

    build_subproject(Path::new("first_stage"), &["_start", "print_char"], &out_dir, &objcopy);
    build_subproject(Path::new("second_stage"), &["second_stage"], &out_dir, &objcopy);
}

fn build_subproject(dir: &Path, global_symbols: &[&str], out_dir: &str, objcopy: &Path) {
    let dir_name = dir.file_name().unwrap().to_str().unwrap();
    let manifest_path = dir.join("Cargo.toml");
    let out_path = Path::new(&out_dir);
    assert!(global_symbols.len() > 0, "must have at least one global symbol");

    // build
    let mut cmd = Command::new("cargo");
    cmd.arg("xbuild").arg("--release");
    cmd.arg(format!("--manifest-path={}", manifest_path.display()));
    cmd.arg("-Z").arg("unstable-options");
    cmd.arg("--out-dir").arg(&out_dir);
    cmd.arg("--target-dir").arg("target");
    cmd.env("XBUILD_SYSROOT_PATH", format!("target/{}-sysroot", dir_name));
    let status = cmd.status().unwrap();
    assert!(status.success());

    // localize symbols
    let mut cmd = Command::new(objcopy);
    for symbol in global_symbols {
        cmd.arg("-G").arg(symbol);
    }
    cmd.arg(out_path.join(format!("lib{}.a", dir_name)));
    let status = cmd.status().unwrap();
    assert!(status.success());

    // emit linker flags
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", dir_name);
}