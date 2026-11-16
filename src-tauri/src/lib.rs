mod seedmap_ext;

use seedmap_core::{
    Dimension, Edition, FeatureQuery, MapFeature, SpawnPoint, TileRequest,
    VersionProfile,
};

#[tauri::command]
fn seedmap_profiles() -> Vec<VersionProfile> {
    seedmap_core::version_profiles()
}

#[tauri::command]
async fn seedmap_render_tile(
    request: TileRequest,
) -> Result<tauri::ipc::Response, String> {
    let pixels = tauri::async_runtime::spawn_blocking(move || {
        seedmap_core::render_tile(request)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;
    Ok(tauri::ipc::Response::new(pixels))
}

#[tauri::command]
async fn seedmap_find_features(
    query: FeatureQuery,
) -> Result<Vec<MapFeature>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        seedmap_core::find_features(query)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

#[tauri::command]
async fn seedmap_spawn(
    seed: String,
    edition: Edition,
    version: String,
) -> Result<SpawnPoint, String> {
    tauri::async_runtime::spawn_blocking(move || {
        seedmap_core::spawn(seed, edition, version)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

#[tauri::command]
async fn seedmap_biome_at(
    seed: String,
    edition: Edition,
    version: String,
    dimension: Dimension,
    x: i32,
    y: i32,
    z: i32,
) -> Result<i32, String> {
    tauri::async_runtime::spawn_blocking(move || {
        seedmap_core::biome_at(seed, edition, version, dimension, x, y, z)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    // macOS keeps the standard app/edit menu so that Cmd+Q, Cmd+C and Cmd+V
    // work inside the webview; other platforms use the plain window instead of
    // a menu bar.
    #[cfg(target_os = "macos")]
    let builder = builder.setup(|app| {
        let menu = tauri::menu::Menu::default(app.handle())?;
        app.set_menu(menu)?;
        Ok(())
    });

    builder
        .invoke_handler(tauri::generate_handler![
            seedmap_profiles,
            seedmap_render_tile,
            seedmap_find_features,
            seedmap_spawn,
            seedmap_biome_at,
            seedmap_ext::seedmap_scan_ores,
            seedmap_ext::seedmap_cancel_ore_scan,
            seedmap_ext::seedmap_read_level_dat,
            seedmap_ext::seedmap_structure_details,
            seedmap_ext::seedmap_container_loot,
            seedmap_ext::seedmap_underground_tile,
            seedmap_ext::seedmap_underground_volume,
            seedmap_ext::seedmap_cancel_underground,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Seedmap Studio");
}
