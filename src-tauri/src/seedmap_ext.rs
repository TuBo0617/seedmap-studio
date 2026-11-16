//! Commands that go beyond the plain map view: ore candidate scans, structure
//! details and loot, underground slices and `level.dat` import.
//!
//! These mirror the ones the upstream Axolotl launcher exposes through its
//! `seed-map` plugin; the only difference is the invoke path (`seedmap_*`
//! instead of `plugin:seed-map|seed_map_*`), because the standalone app does
//! not host a Tauri plugin.

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use seedmap_core::{
    details::{self, StructureQuery, UndergroundRequest},
    ores::{self, OreChunkResult, OreScanRequest},
    underground::{self, VolumeRequest},
    LevelDatInfo,
};

/// Scans are addressed by an ID the caller picks, so a cancel reaches the
/// native loop, and a cancel that arrives before the scan is registered is
/// remembered briefly instead of being lost.
#[derive(Default)]
struct OreScans {
    running: HashMap<u64, bool>,
    early: HashMap<u64, Instant>,
}

const EARLY_ORE_SCAN_CANCEL_TTL: Duration = Duration::from_secs(30);

fn ore_scans() -> &'static Mutex<OreScans> {
    static ORE_SCANS: OnceLock<Mutex<OreScans>> = OnceLock::new();
    ORE_SCANS.get_or_init(Mutex::default)
}

struct OreScan(u64);

impl OreScan {
    fn register(id: u64) -> Self {
        if let Ok(mut scans) = ore_scans().lock() {
            let canceled = scans
                .early
                .remove(&id)
                .is_some_and(|at| at.elapsed() < EARLY_ORE_SCAN_CANCEL_TTL);
            scans.running.insert(id, canceled);
        }
        Self(id)
    }
}

impl Drop for OreScan {
    fn drop(&mut self) {
        if let Ok(mut scans) = ore_scans().lock() {
            scans.running.remove(&self.0);
        }
    }
}

/// Polled by the native scan between veins; an unknown request counts as
/// canceled, since every cancellable scan registers before it starts.
extern "C" fn ore_scan_canceled(id: u64) -> i32 {
    ore_scans().lock().map_or(1, |scans| {
        i32::from(scans.running.get(&id).is_none_or(|canceled| *canceled))
    })
}

fn cancel_ore_scan(id: u64) {
    let Ok(mut scans) = ore_scans().lock() else {
        return;
    };
    if let Some(canceled) = scans.running.get_mut(&id) {
        *canceled = true;
        return;
    }
    scans
        .early
        .retain(|_, at| at.elapsed() < EARLY_ORE_SCAN_CANCEL_TTL);
    if scans.early.len() < 64 {
        scans.early.insert(id, Instant::now());
    }
}

#[tauri::command]
pub async fn seedmap_scan_ores(
    request: OreScanRequest,
    request_id: Option<u64>,
) -> Result<Vec<OreChunkResult>, String> {
    let scan = request_id.map(OreScan::register);
    tauri::async_runtime::spawn_blocking(move || {
        let _scan = scan;
        match request_id {
            Some(id) => ores::scan_ores_cancellable(request, id, ore_scan_canceled),
            None => ores::scan_ores(request),
        }
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn seedmap_cancel_ore_scan(request_id: u64) {
    cancel_ore_scan(request_id);
}

#[tauri::command]
pub async fn seedmap_read_level_dat(path: String) -> Result<LevelDatInfo, String> {
    // Dragging a world folder is the common case; the engine wants the file.
    let path = std::path::PathBuf::from(path);
    let path = if path.is_dir() {
        path.join("level.dat")
    } else {
        path
    };
    tauri::async_runtime::spawn_blocking(move || {
        seedmap_core::read_level_dat(path.to_string_lossy().into_owned())
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn seedmap_structure_details(
    query: StructureQuery,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        details::structure_details(query, None)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn seedmap_container_loot(
    query: StructureQuery,
    container: u32,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        details::structure_details(query, Some(container))
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn seedmap_underground_tile(
    request: UndergroundRequest,
) -> Result<tauri::ipc::Response, String> {
    let pixels = tauri::async_runtime::spawn_blocking(move || {
        details::underground_tile(request)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;
    Ok(tauri::ipc::Response::new(pixels))
}

#[tauri::command]
pub async fn seedmap_underground_volume(
    request: VolumeRequest,
) -> Result<tauri::ipc::Response, String> {
    underground::begin(request.id);
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        underground::generate_volume(request)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;
    Ok(tauri::ipc::Response::new(bytes))
}

#[tauri::command]
pub fn seedmap_cancel_underground(id: u64) {
    underground::cancel(id);
}
