//! Compiles the cubiomes C engine and the FFI bridges into a static library
//! that `src/bridge.rs` links against.
//!
//! The engine is plain C99 with a few GNU extensions (variable length arrays,
//! `__builtin_*` intrinsics), so it is built with the platform compiler as
//! follows:
//!
//! * Linux / macOS - system `cc` (gcc or Apple clang), static archive.
//! * Windows GNU - `x86_64-w64-mingw32-gcc` and friends; this is what the
//!   upstream launcher uses and what CI installs.
//! * Windows MSVC - `clang-cl` when available (clang accepts VLAs), falling
//!   back to `cl.exe`, which rejects VLAs and therefore fails loudly.
//!
//! `vendor/cubiomes/loot/loot_functions.h` carries one local patch: an
//! unnamed `RandomSource*` parameter (C23) is given a name so pre-C23
//! compilers accept the header. See `vendor/cubiomes/PATCHES.md`.

use std::path::{Path, PathBuf};

fn collect_engine_sources(path: &Path, sources: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).expect("Read cubiomes sources") {
        let entry = entry.expect("Read cubiomes entry");
        let path = entry.path();
        if path.is_dir() {
            if matches!(
                entry.file_name().to_str(),
                Some(
                    "features"
                        | "loot"
                        | "loot_tables"
                        | "barrels"
                        | "chests"
                        | "trial_chambers"
                        | "cjson"
                )
            ) {
                collect_engine_sources(&path, sources);
            }
        } else if path.extension().is_some_and(|extension| extension == "c")
            && path.file_name().is_some_and(|name| name != "tests.c")
        {
            sources.push(path);
        }
    }
}

/// True when a compiler with GNU C extensions is available on `PATH`.
fn has_program(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        ["", ".exe", ".cmd", ".bat"]
            .iter()
            .any(|ext| dir.join(format!("{name}{ext}")).is_file())
    })
}

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest.join("vendor/cubiomes");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let is_msvc = target_env == "msvc";

    let mut sources = vec![
        manifest.join("src/cubiomes_bridge.c"),
        manifest.join("src/cubiomes_details.c"),
        manifest.join("src/underground_volume.c"),
        manifest.join("src/ancient_city_layout.c"),
    ];
    collect_engine_sources(&source, &mut sources);
    sources.sort();

    let mut build = cc::Build::new();
    build
        .include(&source)
        .files(&sources)
        .opt_level(3)
        .warnings(false);

    if is_msvc {
        // MSVC's C front end has no VLA support, so prefer clang-cl, which
        // speaks the MSVC command line but accepts the GNU extensions used by
        // the engine.
        if std::env::var_os("CC").is_none() && has_program("clang-cl") {
            build.compiler("clang-cl");
        }
        build
            .flag_if_supported("/std:c11")
            .flag_if_supported("-Wno-everything")
            .define("_CRT_SECURE_NO_WARNINGS", None);
    } else {
        build
            .flag_if_supported("-std=gnu11")
            .flag_if_supported("-fwrapv")
            .flag_if_supported("-Wno-unused-parameter");
    }

    build.compile("cubiomes-engine");

    // libm is folded into libSystem on macOS and does not exist as a separate
    // library on Windows.
    if target_os != "windows" {
        println!("cargo:rustc-link-lib=m");
    }

    println!("cargo:rerun-if-changed=vendor/cubiomes");
    println!("cargo:rerun-if-changed=src/cubiomes_bridge.c");
    println!("cargo:rerun-if-changed=src/cubiomes_bridge.h");
    println!("cargo:rerun-if-changed=src/cubiomes_details.c");
    println!("cargo:rerun-if-changed=src/underground_volume.c");
    println!("cargo:rerun-if-changed=src/ancient_city_layout.c");
    println!("cargo:rerun-if-changed=src/ancient_city_layout.h");
}
