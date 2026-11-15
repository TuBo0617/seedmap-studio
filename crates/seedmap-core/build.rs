//! Compiles the cubiomes C engine and the FFI bridges into a static library
//! that `src/bridge.rs` links against.

use std::path::PathBuf;

fn collect_engine_sources(path: &std::path::Path, sources: &mut Vec<PathBuf>) {
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

fn main() {
    let source = std::path::Path::new("vendor/cubiomes");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let compatibility_header = out.join("cubiomes-loot-functions.h");

    // cubiomes' loot tables use a C23-style unnamed parameter; rewrite the
    // header so older compilers accept it, and point it at the vendored rng.h.
    let header = std::fs::read_to_string(source.join("loot/loot_functions.h"))
        .expect("Read loot header");
    let rng =
        std::fs::canonicalize(source.join("rng.h")).expect("Locate RNG header");
    let header = header
        .replace(
            "RandomSource*, const int min",
            "RandomSource *unused_rnd, const int min",
        )
        .replace(
            "\"../rng.h\"",
            &format!("\"{}\"", rng.display().to_string().replace('\\', "/")),
        );
    std::fs::write(&compatibility_header, header)
        .expect("Write pre-C23 compatibility header");

    let mut sources = vec![
        PathBuf::from("src/cubiomes_bridge.c"),
        PathBuf::from("src/cubiomes_details.c"),
        PathBuf::from("src/underground_volume.c"),
        PathBuf::from("src/ancient_city_layout.c"),
    ];
    collect_engine_sources(source, &mut sources);
    sources.sort();

    cc::Build::new()
        .flag("-include")
        .flag(compatibility_header.to_str().unwrap())
        .include(source)
        .files(&sources)
        .opt_level(3)
        .flag_if_supported("-fwrapv")
        .flag_if_supported("-std=gnu11")
        .warnings(false)
        .compile("axolotl-cubiomes");

    println!("cargo:rustc-link-lib=m");
    println!("cargo:rerun-if-changed=vendor/cubiomes");
    println!("cargo:rerun-if-changed=src/cubiomes_bridge.c");
    println!("cargo:rerun-if-changed=src/cubiomes_bridge.h");
    println!("cargo:rerun-if-changed=src/cubiomes_details.c");
    println!("cargo:rerun-if-changed=src/underground_volume.c");
    println!("cargo:rerun-if-changed=src/ancient_city_layout.c");
}
