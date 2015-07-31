//! A build dependency for running `cmake` to build a native library
//!
//! This crate provides some necessary boilerplate and shim support for running
//! the system `cmake` command to build a native library. It will add
//! appropriate cflags for building code to link into Rust, handle cross
//! compilation, and use the necessary generator for the platform being
//! targeted.
//!
//! The builder-style configuration allows for various variables and such to be
//! passed down into the build as well.
//!
//! ## Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [build-dependencies]
//! cmake = "0.1"
//! ```
//!
//! ## Examples
//!
//! ```no_run
//! use cmake;
//!
//! // Builds the project in the directory located in `libfoo`, installing it
//! // into $OUT_DIR
//! let dst = cmake::build("libfoo");
//!
//! println!("cargo:rustc-link-search=native={}", dst.display());
//! println!("cargo:rustc-link-lib=static=foo");
//! ```
//!
//! ```no_run
//! use cmake::Config;
//!
//! let dst = Config::new("libfoo")
//!                  .define("FOO", "BAR")
//!                  .cflag("-foo")
//!                  .build();
//! println!("cargo:rustc-link-search=native={}", dst.display());
//! println!("cargo:rustc-link-lib=static=foo");
//! ```

#![deny(missing_docs)]

extern crate gcc;

use std::env;
use std::ffi::{OsString, OsStr};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Builder style configuration for a pending CMake build.
pub struct Config {
    path: PathBuf,
    cflags: OsString,
    defines: Vec<(OsString, OsString)>,
    deps: Vec<String>,
}

/// Builds the native library rooted at `path` with the default cmake options.
/// This will return the directory in which the library was installed.
///
/// # Examples
///
/// ```no_run
/// use cmake;
///
/// // Builds the project in the directory located in `libfoo`, installing it
/// // into $OUT_DIR
/// let dst = cmake::build("libfoo");
///
/// println!("cargo:rustc-link-search=native={}", dst.display());
/// println!("cargo:rustc-link-lib=static=foo");
/// ```
///
pub fn build<P: AsRef<Path>>(path: P) -> PathBuf {
    Config::new(path.as_ref()).build()
}

impl Config {
    /// Creates a new blank set of configuration to build the project specified
    /// at the path `path`.
    pub fn new<P: AsRef<Path>>(path: P) -> Config {
        Config {
            path: path.as_ref().to_path_buf(),
            cflags: OsString::new(),
            defines: Vec::new(),
            deps: Vec::new(),
        }
    }

    /// Adds a custom flag to pass down to the compiler, supplementing those
    /// that this library already passes.
    pub fn cflag<P: AsRef<OsStr>>(&mut self, flag: P) -> &mut Config {
        self.cflags.push(" ");
        self.cflags.push(flag.as_ref());
        self
    }

    /// Adds a new `-D` flag to pass to cmake during the generation step.
    pub fn define<K, V>(&mut self, k: K, v: V) -> &mut Config
        where K: AsRef<OsStr>, V: AsRef<OsStr>
    {
        self.defines.push((k.as_ref().to_owned(), v.as_ref().to_owned()));
        self
    }

    /// Registers a dependency for this compilation on the native library built
    /// by Cargo previously.
    ///
    /// This registration will modify the `CMAKE_PREFIX_PATH` environment
    /// variable for the build system generation step.
    pub fn register_dep(&mut self, dep: &str) -> &mut Config {
        self.deps.push(dep.to_string());
        self
    }

    /// Run this configuration, compiling the library with all the configured
    /// options.
    ///
    /// This will run both the build system generator command as well as the
    /// command to build the library.
    pub fn build(&mut self) -> PathBuf {
        let target = env::var("TARGET").unwrap();
        let msvc = target.contains("msvc");

        let dst = PathBuf::from(&env::var("OUT_DIR").unwrap());
        let _ = fs::create_dir(&dst.join("build"));

        // Build up the CFLAGS that we're going to use
        let mut cflags = env::var_os("CFLAGS").unwrap_or(OsString::new());
        cflags.push(" ");
        cflags.push(&self.cflags);
        if !msvc {
            cflags.push(" -ffunction-sections");
            cflags.push(" -fdata-sections");

            if target.contains("i686") {
                cflags.push(" -m32");
            } else if target.contains("x86_64") {
                cflags.push(" -m64");
            }
            if !target.contains("i686") {
                cflags.push(" -fPIC");
            }
        }

        // Add all our dependencies to our cmake paths
        let mut cmake_prefix_path = Vec::new();
        for dep in &self.deps {
            if let Some(root) = env::var_os(format!("DEP_{}_ROOT", dep)) {
                cmake_prefix_path.push(PathBuf::from(root));
            }
        }
        let system_prefix = env::var_os("CMAKE_PREFIX_PATH")
                                .unwrap_or(OsString::new());
        cmake_prefix_path.extend(env::split_paths(&system_prefix)
                                     .map(|s| s.to_owned()));
        let cmake_prefix_path = env::join_paths(&cmake_prefix_path).unwrap();

        // Build up the first cmake command to build the build system.
        let mut cmd = Command::new("cmake");
        cmd.arg(env::current_dir().unwrap().join(&self.path))
           .current_dir(&dst.join("build"));
        if target.contains("windows gnu") {
            cmd.arg("-G").arg("Unix Makefiles");
        } else if msvc {
            if target.contains("i686") {
                cmd.arg("-G").arg("Visual Studio 12 2013");
            } else if target.contains("x86_64") {
                cmd.arg("-G").arg("Visual Studio 12 2013 Win64");
            } else {
                panic!("unsupported msvc target: {}", target);
            }
        }
        let profile = match &env::var("PROFILE").unwrap()[..] {
            "bench" | "release" => "Release",
            _ if msvc => "Release", // currently we need to always use the same CRT
            _ => "Debug",
        };
        for &(ref k, ref v) in &self.defines {
            let mut os = OsString::from("-D");
            os.push(k);
            os.push("=");
            os.push(v);
            cmd.arg(os);
        }
        let mut dstflag = OsString::from("-DCMAKE_INSTALL_PREFIX=");
        dstflag.push(&dst);
        let mut cflagsflag = OsString::from("-DCMAKE_C_FLAGS=");
        cflagsflag.push(&cflags);
        run(cmd.arg(&format!("-DCMAKE_BUILD_TYPE={}", profile))
               .arg(dstflag)
               .arg(cflagsflag)
               .env("CMAKE_PREFIX_PATH", cmake_prefix_path), "cmake");

        // And build!
        run(Command::new("cmake")
                    .arg("--build").arg(".")
                    .arg("--target").arg("install")
                    .arg("--config").arg(profile)
                    .current_dir(&dst.join("build")), "cmake");

        println!("cargo:root={}", dst.display());
        return dst
    }
}

fn run(cmd: &mut Command, program: &str) {
    println!("running: {:?}", cmd);
    let status = match cmd.status() {
        Ok(status) => status,
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            fail(&format!("failed to execute command: {}\nis `{}` not installed?",
                          e, program));
        }
        Err(e) => fail(&format!("failed to execute command: {}", e)),
    };
    if !status.success() {
        fail(&format!("command did not execute successfully, got: {}", status));
    }
}

fn fail(s: &str) -> ! {
    panic!("\n{}\n\nbuild script failed, must exit now", s)
}
