use std::{
    path::PathBuf,
    sync::{mpsc::Sender, Arc, Mutex},
};

use crate::dat_descriptor::DatDescriptor;
use dats::context::DatContext;
use serde::{Deserialize, Serialize};
use threadpool::ThreadPool;

#[derive(Debug)]
pub struct DatProcessor {
    tx: Sender<DatProcessorMessage>,
    pub is_preprocessing: bool,
    pool: Mutex<ThreadPool>,
}

#[derive(Debug, Clone, specta::Type, Serialize, Deserialize)]
pub struct DatProcessorMessage {
    pub dat_descriptor: DatDescriptor,
    pub output_kind: DatProcessorOutputKind,
    pub state: DatProcessingState,
}

#[derive(Debug, Clone, specta::Type, Serialize, Deserialize)]
pub enum DatProcessorOutputKind {
    Dat,
    Yaml,
}

#[derive(Debug, Clone, specta::Type, Serialize, Deserialize)]
pub enum DatProcessingState {
    Working,
    Finished(PathBuf),
    Error(String),
}

impl DatProcessor {
    pub fn new(tx: Sender<DatProcessorMessage>) -> Self {
        Self {
            tx,
            is_preprocessing: false,
            pool: Mutex::new(
                threadpool::Builder::new()
                    .thread_name("dat-processor".to_string())
                    .build(),
            ),
        }
    }

    pub fn dat_to_yaml(
        &self,
        dat_descriptor: DatDescriptor,
        dat_context: Arc<DatContext>,
        raw_data_root_path: PathBuf,
    ) {
        let tx = self.tx.clone();
        let start_message = DatProcessorMessage {
            dat_descriptor,
            output_kind: DatProcessorOutputKind::Yaml,
            state: DatProcessingState::Working,
        };
        if let Err(err) = tx.send(start_message) {
            eprintln!("Failed to notify about DAT to YAML start: {err}");
        }

        self.pool.lock().unwrap().execute(move || {
            let res = dat_descriptor
                .dat_to_yaml(dat_context, raw_data_root_path)
                .map(|path| DatProcessorMessage {
                    dat_descriptor,
                    output_kind: DatProcessorOutputKind::Yaml,
                    state: DatProcessingState::Finished(path),
                })
                .unwrap_or_else(|err| DatProcessorMessage {
                    dat_descriptor,
                    output_kind: DatProcessorOutputKind::Yaml,
                    state: DatProcessingState::Error(err.to_string()),
                });

            if let Err(err) = tx.send(res) {
                eprintln!("Failed to notify about DAT to YAML result: {err}");
            }
        });
    }

    pub fn yaml_to_dat(
        &self,
        dat_descriptor: DatDescriptor,
        dat_context: Arc<DatContext>,
        raw_data_root_path: PathBuf,
        dat_root_path: PathBuf,
    ) {
        let tx = self.tx.clone();
        let start_message = DatProcessorMessage {
            dat_descriptor,
            output_kind: DatProcessorOutputKind::Dat,
            state: DatProcessingState::Working,
        };
        if let Err(err) = tx.send(start_message) {
            eprintln!("Failed to notify about YAML to DAT start: {err}");
        }

        self.pool.lock().unwrap().execute(move || {
            let res: DatProcessorMessage = dat_descriptor
                .yaml_to_dat(dat_context, raw_data_root_path, dat_root_path)
                .map(|path| DatProcessorMessage {
                    dat_descriptor,
                    output_kind: DatProcessorOutputKind::Dat,
                    state: DatProcessingState::Finished(path),
                })
                .unwrap_or_else(|err| DatProcessorMessage {
                    dat_descriptor,
                    output_kind: DatProcessorOutputKind::Dat,
                    state: DatProcessingState::Error(err.to_string()),
                });

            if let Err(err) = tx.send(res) {
                eprintln!("Failed to notify about YAML to DAT result: {err}");
            }
        });
    }

    pub fn all_yaml_to_dats(
        &mut self,
        dat_context: Arc<DatContext>,
        in_dir: &PathBuf,
        out_dir: &PathBuf,
    ) -> usize {
        self.is_preprocessing = true;

        let mut count = 0;

        walkdir::WalkDir::new(&in_dir)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().is_dir() {
                    return None;
                }

                let path = entry.into_path();
                let dat_descriptor = DatDescriptor::from_path(&path, &in_dir, &dat_context);

                if dat_descriptor.is_none() {
                    eprintln!(
                        "Could not map the following file to a DAT: {}",
                        path.to_string_lossy()
                    );
                }
                dat_descriptor
            })
            .for_each(|dat_descriptor| {
                let dat_context = dat_context.clone();
                let raw_data_root_path = in_dir.clone();
                let dat_root_path = out_dir.clone();
                count += 1;

                self.yaml_to_dat(
                    dat_descriptor,
                    dat_context,
                    raw_data_root_path,
                    dat_root_path,
                );
            });

        self.is_preprocessing = false;
        count
    }
}
