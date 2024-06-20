// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_persistence;
mod cli;
mod commands;
mod dat_query;
mod errors;
mod state;

use cli::check_cli;
use parking_lot::RwLock;
use state::AppStateData;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

// #[cfg(debug_assertions)]
// use specta::collect_types;
// #[cfg(debug_assertions)]
// use tauri_specta::ts;

pub const RAW_DATA_DIR: &'static str = "raw_data";
pub const LOOKUP_TABLE_DIR: &'static str = "lookup_tables";
pub const DAT_GENERATION_DIR: &'static str = "generated_dats";
pub const ZONE_MAPPING_FILE: &'static str = "zones.yml";

fn main() {
    check_cli();

    let speta_builder = {
        let builder = tauri_specta::ts::builder().commands(tauri_specta::collect_commands![
            commands::dummy_event_type_gen,
            commands::select_ffxi_folder,
            commands::select_project_folder,
            commands::load_persistence_data,
            commands::get_misc_dats,
            commands::get_standalone_string_dats,
            commands::get_item_dats,
            commands::get_global_dialog_dats,
            commands::browse_dats,
            commands::get_zones_for_type,
            commands::get_working_files,
            commands::make_all_dats,
            commands::make_dat,
            commands::make_yaml,
            commands::copy_lookup_tables,
        ]);

        #[cfg(debug_assertions)]
        let builder = builder.path("../src/bindings.ts").header("// @ts-nocheck");

        builder.build().unwrap()
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(speta_builder)
        .invoke_handler(tauri::generate_handler![
            commands::select_ffxi_folder,
            commands::select_project_folder,
            commands::load_persistence_data,
            commands::browse_dats,
            commands::get_zones_for_type,
            commands::get_misc_dats,
            commands::get_standalone_string_dats,
            commands::get_item_dats,
            commands::get_global_dialog_dats,
            commands::get_working_files,
            commands::make_all_dats,
            commands::make_dat,
            commands::make_yaml,
            commands::copy_lookup_tables,
        ])
        .setup(|app| {
            let app_state = RwLock::new(AppStateData::new(app));
            app.manage(app_state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
