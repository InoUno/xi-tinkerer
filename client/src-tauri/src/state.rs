use std::{
    ffi::OsStr,
    path::PathBuf,
    sync::{mpsc, Arc},
    thread,
};

use anyhow::Result;
use dats::context::DatContext;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use processor::{dat_descriptor::DatDescriptor, processor::DatProcessor};
use serde::Serialize;
use tauri::{async_runtime, App, AppHandle, Manager};

use crate::{app_persistence::PersistenceData, errors::AppError, RAW_DATA_DIR};

#[derive(Debug)]
pub struct AppStateData {
    pub project_path: Option<PathBuf>,
    pub dat_context: Option<Arc<DatContext>>,
    pub processor: Arc<DatProcessor>,
    pub persistence: PersistenceData,
    watcher: RecommendedWatcher,
    local_data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct FileNotification {
    dat_descriptor: DatDescriptor,
    is_delete: bool,
}

impl AppStateData {
    pub fn new(app: &App) -> Self {
        let local_data_dir = app.path().local_data_dir().unwrap();
        let persistence = PersistenceData::load(&local_data_dir);

        let dat_context = persistence
            .ffxi_path
            .as_ref()
            .and_then(|ffxi_path| DatContext::from_ffxi_path(ffxi_path.clone()).ok())
            .map(|context| Arc::new(context));

        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx).unwrap();

        let handle = app.handle().clone();
        thread::spawn(move || Self::watch_handler(rx, handle));

        let project_path = persistence.recent_projects.get(0).cloned();

        if let Some(project_path) = &project_path {
            let _ = watcher.watch(&project_path, RecursiveMode::Recursive);
        }

        let (tx, rx) = mpsc::channel();
        let processor = Arc::new(DatProcessor::new(tx));

        let app_handle = app.handle().clone();
        async_runtime::spawn(async move {
            while let Ok(msg) = rx.recv() {
                if let Err(err) = app_handle.emit("processing", msg) {
                    eprintln!("Failed to emit message to all: {err}");
                }
            }
        });

        Self {
            dat_context,
            project_path,
            persistence,
            watcher,
            processor,
            local_data_dir,
        }
    }

    pub fn set_ffxi_path(
        &mut self,
        ffxi_path: Option<PathBuf>,
    ) -> Result<Option<PathBuf>, AppError> {
        let context = if let Some(ffxi_path) = ffxi_path {
            Some(Arc::new(DatContext::from_ffxi_path(ffxi_path)?))
        } else {
            None
        };

        let new_ffxi_path = context.as_ref().map(|context| context.ffxi_path.clone());

        self.dat_context = context;
        self.persistence.ffxi_path = new_ffxi_path.clone();
        self.persistence.save(&self.local_data_dir);

        Ok(new_ffxi_path)
    }

    pub fn set_project_path(
        &mut self,
        project_path: Option<PathBuf>,
    ) -> Result<Vec<PathBuf>, AppError> {
        // Remove previous path from being watched
        if let Some(previous_path) = &self.project_path {
            let _ = self.watcher.unwatch(previous_path);
        }

        self.project_path = project_path.clone();

        if let Some(project_path) = project_path {
            // Remove project from list if it's already there, and then insert it at the front
            let filtered_recent_project = self
                .persistence
                .recent_projects
                .iter()
                .filter(|project| *project != &project_path);

            self.persistence.recent_projects = std::iter::once(&project_path)
                .chain(filtered_recent_project)
                .take(5)
                .cloned()
                .collect();

            self.persistence.save(&self.local_data_dir);

            // Start watching new project data directory
            let _ = self.watcher.watch(&project_path, RecursiveMode::Recursive);
        }

        Ok(self.persistence.recent_projects.clone())
    }

    fn watch_handler(rx: std::sync::mpsc::Receiver<notify::Result<Event>>, app_handle: AppHandle) {
        while let Ok(event) = rx.recv() {
            match event {
                Ok(event) => {
                    let _ = Self::handle_file_event(event, app_handle.clone());
                }
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            }
        }
    }

    fn handle_file_event(event: Event, app_handle: AppHandle) -> Option<()> {
        // Only send event on creation and removal of files.
        let is_delete = match event.kind {
            EventKind::Create(_) => false,
            EventKind::Remove(_) => true,
            _ => return None,
        };

        let app_state: AppState = app_handle.state();
        let project_path = app_state.read().project_path.clone().unwrap();

        let raw_data_paths = event
            .paths
            .into_iter()
            .filter_map(|path| {
                path.strip_prefix(&project_path.join(RAW_DATA_DIR))
                    .map(|p| p.to_path_buf())
                    .ok()
            })
            .collect::<Vec<_>>();

        for path in raw_data_paths {
            if let Some(dat_descriptor) = Self::get_file_dat_descriptor(&path, &app_state) {
                let notification = FileNotification {
                    dat_descriptor,
                    is_delete,
                };
                let _ = app_handle.emit("file-change", notification);
            }
        }

        Some(())
    }

    fn get_file_dat_descriptor(path: &PathBuf, app_state: &AppState) -> Option<DatDescriptor> {
        if path.is_dir() || path.extension() != Some(OsStr::new("yml")) {
            return None;
        }

        let dat_context = app_state.read().dat_context.clone()?;
        let raw_data_dir = app_state.read().project_path.clone()?.join(RAW_DATA_DIR);

        DatDescriptor::from_path(path, &raw_data_dir, &dat_context)
    }
}

pub type AppState<'a> = tauri::State<'a, RwLock<AppStateData>>;
