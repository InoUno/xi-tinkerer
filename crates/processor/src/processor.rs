use std::{
    path::PathBuf,
    sync::{mpsc::SyncSender, Arc, Mutex},
};

use crate::dat_descriptor::DatDescriptor;
use dats::context::DatContext;
use serde::{Deserialize, Serialize};
use threadpool::ThreadPool;

#[derive(Debug)]
pub struct DatProcessor {
    tx: SyncSender<DatProcessorMessage>,
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
    Finished,
    Error(String),
}

impl DatProcessor {
    pub fn new(tx: SyncSender<DatProcessorMessage>) -> Self {
        Self {
            tx,
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
                .map(|_| DatProcessorMessage {
                    dat_descriptor,
                    output_kind: DatProcessorOutputKind::Yaml,
                    state: DatProcessingState::Finished,
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
            let res = dat_descriptor
                .yaml_to_dat(dat_context, raw_data_root_path, dat_root_path)
                .map(|_| DatProcessorMessage {
                    dat_descriptor,
                    output_kind: DatProcessorOutputKind::Dat,
                    state: DatProcessingState::Finished,
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
}
