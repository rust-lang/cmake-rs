extern crate tempfile;

use std::{env, error::Error, fs, path::Path};

#[test]
fn handling_reset_cache_after_set_var() {
    let tmp_dir = tempfile::tempdir().unwrap();

    fill_dir(tmp_dir.path()).unwrap();

    if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_vendor = "unknown"
    )) {
        env::set_var("TARGET", "x86_64-unknown-linux-gnu");
        env::set_var("HOST", "x86_64-unknown-linux-gnu");
        env::set_var("OUT_DIR", tmp_dir.path());
        env::set_var("PROFILE", "debug");
        env::set_var("OPT_LEVEL", "0");
        env::set_var("DEBUG", "true");
        let dst = cmake::Config::new(tmp_dir.path())
            .define("OPT1", "False")
            .build_target("all")
            .build()
            .join("build");

        assert!(fs::read_to_string(dst.join("CMakeCache.txt"))
            .unwrap()
            .contains("OPT1:BOOL=False"));
        env::set_var("CC", "clang");
        env::set_var("CXX", "clang++");
        let dst = cmake::Config::new(tmp_dir.path())
            .define("OPT1", "False")
            .build_target("all")
            .build()
            .join("build");

        assert!(fs::read_to_string(dst.join("CMakeCache.txt"))
            .unwrap()
            .contains("OPT1:BOOL=False"));
    }
}

fn fill_dir(tmp_dir: &Path) -> Result<(), Box<dyn Error>> {
    fs::write(
        tmp_dir.join("CMakeLists.txt"),
        r#"
project(xyz)
cmake_minimum_required(VERSION 3.9)

option(OPT1 "some option" ON)
add_executable(xyz main.cpp)
"#,
    )?;
    fs::write(
        tmp_dir.join("main.cpp"),
        r#"
#include <cstdio>
#define DO_STRINGIFY(x) #x
#define STRINGIFY(x) DO_STRINGIFY(x)
int main() {
	printf("option: %s\n", STRINGIFY(OPT1));
}
"#,
    )?;
    Ok(())
}
