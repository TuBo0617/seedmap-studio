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
//! * Android - the NDK's LLVM toolchain (`<triple><api>-clang`), one static
//!   archive per ABI (arm64-v8a, armeabi-v7a, x86, x86_64).
//! * iOS - Apple clang against the iPhoneOS / iPhoneSimulator SDK, with the
//!   deployment target pinned by `IPHONEOS_DEPLOYMENT_TARGET`.
//!
//! `vendor/cubiomes/loot/loot_functions.h` carries one local patch: an
//! unnamed `RandomSource*` parameter (C23) is given a name so pre-C23
//! compilers accept the header. See `vendor/cubiomes/PATCHES.md`.

use std::path::{Path, PathBuf};

/// Minimum API level the Android build targets. 24 (Android 7.0) keeps the
/// engine on 64-bit capable devices back to 2016 while staying inside the
/// range supported by the NDK and by Tauri.
const ANDROID_API: u32 = 24;

/// Deployment target floor for iOS. Tauri 2 / wry rely on `UIWindowScene`,
/// which is only available from iOS 13, so this cannot go lower.
const IOS_DEPLOYMENT_TARGET: &str = "13.0";

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

/// Locates the Android NDK. Tauri's CLI exports `NDK_HOME` when it shells out
/// to cargo, but a plain `cargo build --target ...-linux-android` does not,
/// so the well known variables plus the SDK layout are checked as fallbacks.
fn find_android_ndk() -> Option<PathBuf> {
    for key in [
        "NDK_HOME",
        "ANDROID_NDK_HOME",
        "ANDROID_NDK_ROOT",
        "ANDROID_NDK_PATH",
    ] {
        if let Some(value) = std::env::var_os(key) {
            let path = PathBuf::from(value);
            if path.is_dir() {
                return Some(path);
            }
        }
    }

    for key in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        let Some(root) = std::env::var_os(key) else {
            continue;
        };
        let root = PathBuf::from(root);
        let bundle = root.join("ndk-bundle");
        if bundle.is_dir() {
            return Some(bundle);
        }
        // Pick the highest installed side-by-side NDK.
        let ndk_root = root.join("ndk");
        let mut candidates: Vec<PathBuf> = std::fs::read_dir(&ndk_root)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect();
        candidates.sort();
        if let Some(latest) = candidates.pop() {
            return Some(latest);
        }
    }

    None
}

fn android_toolchain(target_arch: &str) -> (PathBuf, String, u32) {
    let ndk = find_android_ndk().unwrap_or_else(|| {
        panic!(
            "Android NDK not found. Set NDK_HOME or ANDROID_NDK_HOME, or install \
             the NDK through the Android SDK manager."
        )
    });

    // The NDK ships a prebuilt LLVM for the build host only.
    let host_tag = if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "darwin-x86_64"
        } else {
            "darwin-x86_64"
        }
    } else if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else {
        "linux-x86_64"
    };
    let bin = ndk.join("toolchains/llvm/prebuilt").join(host_tag).join("bin");

    // `armv7-linux-androideabi` is the Rust triple; the NDK spells the same
    // ABI `armv7a-linux-androideabi` and also offers a thumb variant.
    let clang_triple = match target_arch {
        "aarch64" => "aarch64-linux-android",
        "arm" => "armv7a-linux-androideabi",
        "x86" => "i686-linux-android",
        "x86_64" => "x86_64-linux-android",
        other => panic!("unsupported Android architecture: {other}"),
    };

    (bin, clang_triple.to_string(), ANDROID_API)
}

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest.join("vendor/cubiomes");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let is_msvc = target_env == "msvc";
    let is_apple_mobile = matches!(target_os.as_str(), "ios" | "tvos" | "watchos" | "visionos");

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

    if target_os == "android" {
        let (bin, triple, api) = android_toolchain(&target_arch);
        let compiler = bin.join(format!("{triple}{api}-clang"));
        assert!(
            compiler.is_file(),
            "Android NDK clang wrapper missing: {}",
            compiler.display()
        );
        let api_string = api.to_string();
        // The wrapper already knows its target triple and sysroot; the
        // explicit flag keeps things working when a generic clang is picked
        // up instead.
        build
            .compiler(compiler)
            .flag(format!("--target={triple}{api}"))
            .flag_if_supported("-std=gnu11")
            .flag_if_supported("-fwrapv")
            .flag_if_supported("-Wno-unused-parameter")
            .flag_if_supported("-Wno-everything")
            .pic(true)
            .define("__ANDROID_API__", Some(api_string.as_str()));
    } else if is_apple_mobile {
        // Xcode's clang is picked automatically on macOS; pinning the
        // deployment target here keeps the static archive usable on the
        // oldest supported OS. Simulator and device SDKs want different
        // spellings of the same flag.
        let target = std::env::var("TARGET").unwrap_or_default();
        let (sdk, minimum) = if target.contains("sim") {
            ("-mios-simulator-version-min", IOS_DEPLOYMENT_TARGET)
        } else {
            ("-mios-version-min", IOS_DEPLOYMENT_TARGET)
        };
        build
            .flag_if_supported("-std=gnu11")
            .flag_if_supported("-fwrapv")
            .flag_if_supported("-Wno-unused-parameter")
            .flag(format!("{sdk}={minimum}"))
            .pic(true);
    } else if is_msvc {
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

    // libm is folded into libSystem on Apple platforms, does not exist on
    // Windows, and is a real (if tiny) shared library on Android.
    if !matches!(target_os.as_str(), "windows" | "ios" | "tvos" | "watchos" | "visionos") {
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
