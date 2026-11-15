//! FFI surface of the C engine.
//!
//! `build.rs` compiles cubiomes and the bridges into a static archive, so the
//! symbols below are linked into the binary on every platform. Upstream ships
//! the engine as a DLL on Windows and loads it at runtime; a standalone app has
//! no reason to do that, and static linking keeps the ABI check in one place.

use super::BridgeFeature;
use std::ffi::c_char;

/// The engine is part of the executable, so it is always available.
pub(super) fn ensure_available() -> std::io::Result<()> {
    Ok(())
}

macro_rules! engine_function {
    ($name:ident($($argument:ident: $ty:ty),* $(,)?) -> $result:ty, $failure:expr) => {
        unsafe extern "C" {
            pub(super) fn $name($($argument: $ty),*) -> $result;
        }
    };
}

engine_function!(axolotl_seed_map_java_version(version: i32) -> i32, -1);
engine_function!(axolotl_seed_map_render(
        seed: u64,
        minecraft_version: i32,
        generator_flags: i32,
        dimension: i32,
        x: i32,
        z: i32,
        scale: i32,
        width: i32,
        height: i32,
        elevation: i32,
        terrain: i32,
        contours: i32,
        highlight_mask: *const u8,
        rgb: *mut u8,
        rgb_len: usize,
    ) -> i32, -1);
engine_function!(axolotl_seed_map_find_features(
        seed: u64,
        minecraft_version: i32,
        generator_flags: i32,
        dimension: i32,
        min_x: i32,
        min_z: i32,
        max_x: i32,
        max_z: i32,
        feature_mask: u32,
        out: *mut BridgeFeature,
        out_len: usize,
    ) -> usize, 0);
engine_function!(axolotl_seed_map_get_spawn(
        seed: u64,
        minecraft_version: i32,
        generator_flags: i32,
        x: *mut i32,
        z: *mut i32,
    ) -> i32, -1);
engine_function!(axolotl_seed_map_biome_at(
        seed: u64,
        minecraft_version: i32,
        generator_flags: i32,
        dimension: i32,
        x: i32,
        y: i32,
        z: i32,
    ) -> i32, -1);

engine_function!(axolotl_seed_map_scan_ores(seed: u64, mc: i32, flags: i32, dimension: i32, cx: i32, cz: i32, kind: i32, xyz: *mut i32, capacity: usize) -> usize, 0);
engine_function!(axolotl_seed_map_scan_ores_cancellable(seed: u64, mc: i32, flags: i32, dimension: i32, cx: i32, cz: i32, kind: i32, xyz: *mut i32, capacity: usize, request_id: u64, canceled: extern "C" fn(u64) -> i32) -> usize, 0);

engine_function!(axolotl_seed_map_capability(mc: i32, capability: i32, value: i32) -> i32, 0);

engine_function!(axolotl_seed_map_structure_details(seed: u64, mc: i32, flags: i32, dimension: i32, x: i32, z: i32, feature: u32, container: i32) -> *mut c_char, std::ptr::null_mut());
engine_function!(axolotl_seed_map_ancient_city_layout(seed: u64, mc: i32, x: i32, z: i32, layout_json: *const c_char, layout_len: usize) -> *mut c_char, std::ptr::null_mut());
engine_function!(axolotl_seed_map_generate_loot(table_json: *const c_char, table_len: usize, seed: u64, mc: i32) -> *mut c_char, std::ptr::null_mut());
engine_function!(axolotl_seed_map_free_result(result: *mut c_char) -> (), ());
engine_function!(axolotl_seed_map_underground(seed: u64, mc: i32, flags: i32, dimension: i32, x: i32, z: i32, scale: i32, width: i32, height: i32, y: i32, layers: u32, rgba: *mut u8, length: u64) -> i32, -1);
engine_function!(axolotl_seed_map_underground_volume(seed: u64, mc: i32, flags: i32, dimension: i32, x: i32, z: i32, y: i32, width: i32, height: i32, blocks: *mut u8, length: u64) -> i32, -1);
engine_function!(axolotl_seed_map_underground_chunk(seed: u64, mc: i32, flags: i32, dimension: i32, x: i32, z: i32, blocks: *mut u8, length: u64, surface_heights: *mut i32, request_id: u64, canceled: extern "C" fn(u64) -> i32) -> i32, -1);
