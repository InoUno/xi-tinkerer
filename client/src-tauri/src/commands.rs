use std::path::PathBuf;

use anyhow::{anyhow, Result};
use processor::{dat_descriptor::DatDescriptor, processor::DatProcessorMessage};

use crate::{
    app_persistence::PersistenceData,
    dat_query,
    dat_query::ZoneInfo,
    errors::AppError,
    state::{AppState, FileNotification},
    {DAT_GENERATION_DIR, RAW_DATA_DIR},
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

// Dummy command just to create types for events
#[tauri::command]
#[specta::specta]
#[allow(unused)]
pub async fn dummy_event_type_gen() -> Result<(FileNotification, DatProcessorMessage), AppError> {
    Err(anyhow!("N/A"))?
}
