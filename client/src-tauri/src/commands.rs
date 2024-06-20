use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use processor::{dat_descriptor::DatDescriptor, processor::DatProcessorMessage};
use tracing_subscriber::fmt::MakeWriter;

use crate::{
    app_persistence::PersistenceData,
    dat_query::{self, BrowseInfo, ZoneInfo},
    errors::AppError,
    state::{AppState, FileNotification},
    DAT_GENERATION_DIR, LOOKUP_TABLE_DIR, RAW_DATA_DIR, ZONE_MAPPING_FILE,
};

#[tauri::command]
#[specta::specta]
pub async fn select_ffxi_folder<'a>(
    path: Option<PathBuf>,
    state: AppState<'a>,
) -> Result<Option<PathBuf>, AppError> {
    state.write().set_ffxi_path(path)
}

#[tauri::command]
#[specta::specta]
pub async fn select_project_folder<'a>(
    path: Option<PathBuf>,
    state: AppState<'a>,
) -> Result<Vec<PathBuf>, AppError> {
    state.write().set_project_path(path)
}

#[tauri::command]
#[specta::specta]
pub async fn load_persistence_data<'a>(state: AppState<'a>) -> Result<PersistenceData, AppError> {
    Ok(state.read().persistence.clone())
}

#[tauri::command]
#[specta::specta]
pub async fn browse_dats(state: AppState<'_>) -> Result<Vec<BrowseInfo>, AppError> {
    let dat_context = state
        .read()
        .dat_context
        .clone()
        .ok_or(anyhow!("No DAT context."))?;

    Ok(dat_query::get_browse_info(dat_context).await)
}

#[tauri::command]
#[specta::specta]
pub async fn get_zones_for_type(
    dat_descriptor: DatDescriptor,
    state: AppState<'_>,
) -> Result<Vec<ZoneInfo>, AppError> {
    let dat_context = state
        .read()
        .dat_context
        .clone()
        .ok_or(anyhow!("No DAT context."))?;

    Ok(dat_query::get_zone_ids_for_type(dat_descriptor, dat_context).await)
}

#[tauri::command]
#[specta::specta]
pub async fn get_misc_dats() -> Result<Vec<DatDescriptor>, AppError> {
    Ok(dat_query::get_misc_dats())
}

#[tauri::command]
#[specta::specta]
pub async fn get_standalone_string_dats() -> Result<Vec<DatDescriptor>, AppError> {
    Ok(dat_query::get_standalone_string_dats())
}

#[tauri::command]
#[specta::specta]
pub async fn get_item_dats() -> Result<Vec<DatDescriptor>, AppError> {
    Ok(dat_query::get_item_dats())
}

#[tauri::command]
#[specta::specta]
pub async fn get_global_dialog_dats() -> Result<Vec<DatDescriptor>, AppError> {
    Ok(dat_query::get_global_dialog_dats())
}

#[tauri::command]
#[specta::specta]
pub async fn get_working_files(state: AppState<'_>) -> Result<Vec<DatDescriptor>, AppError> {
    let dat_context = state
        .read()
        .dat_context
        .clone()
        .ok_or(anyhow!("No DAT context."))?;

    let project_path = state
        .read()
        .project_path
        .as_ref()
        .ok_or(anyhow!("No project path specified."))?
        .clone();

    let raw_data_dir = project_path.join(RAW_DATA_DIR);
    Ok(walkdir::WalkDir::new(&raw_data_dir)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            DatDescriptor::from_path(&entry.into_path(), &raw_data_dir, &dat_context)
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub async fn make_all_dats(state: AppState<'_>) -> Result<(), AppError> {
    let dat_context = state
        .read()
        .dat_context
        .clone()
        .ok_or(anyhow!("No DAT context."))?;

    let project_path = state
        .read()
        .project_path
        .as_ref()
        .ok_or(anyhow!("No project path specified."))?
        .clone();

    let processor = state.read().processor.clone();

    let raw_data_dir = project_path.join(RAW_DATA_DIR);
    let dat_root_path = project_path.join(DAT_GENERATION_DIR);

    Ok(walkdir::WalkDir::new(&raw_data_dir)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            DatDescriptor::from_path(&entry.into_path(), &raw_data_dir, &dat_context)
        })
        .for_each(|dat_descriptor| {
            let dat_context = dat_context.clone();
            let raw_data_root_path = raw_data_dir.clone();
            let dat_root_path = dat_root_path.clone();
            processor.yaml_to_dat(
                dat_descriptor,
                dat_context,
                raw_data_root_path,
                dat_root_path,
            );
        }))
}

#[tauri::command]
#[specta::specta]
pub async fn make_dat(dat_descriptor: DatDescriptor, state: AppState<'_>) -> Result<(), AppError> {
    let dat_context = state
        .read()
        .dat_context
        .clone()
        .ok_or(anyhow!("No DAT context."))?;

    let project_path = state
        .read()
        .project_path
        .as_ref()
        .ok_or(anyhow!("No project path specified."))?
        .clone();

    let processor = state.read().processor.clone();

    processor.yaml_to_dat(
        dat_descriptor,
        dat_context,
        project_path.join(RAW_DATA_DIR),
        project_path.join(DAT_GENERATION_DIR),
    );

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn make_yaml(dat_descriptor: DatDescriptor, state: AppState<'_>) -> Result<(), AppError> {
    let dat_context = state
        .read()
        .dat_context
        .clone()
        .ok_or(anyhow!("No DAT context."))?;

    let project_path = state
        .read()
        .project_path
        .as_ref()
        .ok_or(anyhow!("No project path specified."))?
        .clone();

    let processor = state.read().processor.clone();

    processor.dat_to_yaml(dat_descriptor, dat_context, project_path.join(RAW_DATA_DIR));

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn copy_lookup_tables(state: AppState<'_>) -> Result<(), AppError> {
    let dat_context = state
        .read()
        .dat_context
        .clone()
        .ok_or(anyhow!("No DAT context."))?;

    let mut lookup_table_dir = state
        .read()
        .project_path
        .as_ref()
        .ok_or(anyhow!("No project path specified."))?
        .clone();
    lookup_table_dir.push(LOOKUP_TABLE_DIR);

    // Clean old lookup table directory, if it already exists.
    if lookup_table_dir.exists() {
        fs::remove_dir_all(&lookup_table_dir)
            .map_err(|err| anyhow!("Unable to clear out old lookup tables: {}", err))?;
    }

    // Helper function for copying a file and checking for errors
    let copy_lookup_table = |sub_path: &str| -> Result<(), AppError> {
        let to_path = lookup_table_dir.join(sub_path);
        fs::create_dir_all(&to_path.parent().unwrap()).map_err(|err| {
            anyhow!(
                "Unable to create directory for lookup table '{}': {}",
                to_path.to_string_lossy(),
                err
            )
        })?;

        let from_path = dat_context.ffxi_path.join(sub_path);
        fs::copy(&from_path, lookup_table_dir.join(sub_path)).map_err(|err| {
            anyhow!(
                "Unable to copy lookup table file '{}': {}",
                from_path.to_string_lossy(),
                err
            )
        })?;

        Ok(())
    };

    // Handle first non-numbered tables
    copy_lookup_table("VTABLE.DAT")?;
    copy_lookup_table("FTABLE.DAT")?;

    // Handle remaining numbered tables in each corresponding ROM folder
    for rom_id in 2u8.. {
        let vtable_sub_path = format!("ROM{}/VTABLE{}.DAT", rom_id, rom_id);

        let vtable_dat_path = dat_context.ffxi_path.join(&vtable_sub_path);
        if !vtable_dat_path.exists() {
            // Break out when no more lookup tables can be found
            break;
        }
        copy_lookup_table(&vtable_sub_path)?;

        let ftable_sub_path = format!("ROM{}/FTABLE{}.DAT", rom_id, rom_id);
        copy_lookup_table(&ftable_sub_path)?;
    }

    // Dump zones mapping file
    let zone_map_file = lookup_table_dir.join(ZONE_MAPPING_FILE);
    let zone_file = File::create(zone_map_file)
        .map_err(|err| anyhow!("Unable to open zone mapping file: {}", err))?;

    let sorted_zones: BTreeMap<_, _> = dat_context.zone_id_to_name.iter().collect();
    serde_yaml::to_writer(zone_file.make_writer(), &sorted_zones)
        .map_err(|err| anyhow!("Unable to write zone mapping file: {}", err))?;

    Ok(())
}

// Dummy command just to create types for events
#[tauri::command]
#[specta::specta]
#[allow(unused)]
pub async fn dummy_event_type_gen() -> Result<(FileNotification, DatProcessorMessage), AppError> {
    Err(anyhow!("N/A"))?
}
