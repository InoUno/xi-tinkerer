use std::{
    collections::HashMap,
    fs::File,
    path::PathBuf,
    str::FromStr,
    sync::{mpsc, Arc},
};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use dats::context::{DatContext, ZoneName};
use processor::processor::{DatProcessingState, DatProcessor};

use crate::{DAT_GENERATION_DIR, LOOKUP_TABLE_DIR, RAW_DATA_DIR, ZONE_MAPPING_FILE};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ExportDats {
        #[arg(value_name = "PROJECT_DIR")]
        project_dir: String,
    },
}

fn attach_console() {
    #[cfg(windows)]
    {
        use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        let _ = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
    }
}

pub fn check_cli() {
    if std::env::args_os().len() > 1 {
        // Ensure a console is attached on Windows
        attach_console();
    }

    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Commands::ExportDats { project_dir } => {
                export_all_dats(project_dir).unwrap();
            }
        }

        std::process::exit(0);
    }
}

pub fn export_all_dats(project_dir: String) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let mut processor = DatProcessor::new(tx);

    let project_path = PathBuf::from_str(&project_dir)?;
    println!("Processing project: {}", project_dir);

    let lookup_dir = project_path.join(LOOKUP_TABLE_DIR);

    // Load zone mapping
    let zone_map_file = lookup_dir.join(ZONE_MAPPING_FILE);
    let zone_file = File::open(zone_map_file)
        .map_err(|err| anyhow!("Unable to open zone mapping file: {}", err))?;
    let zones_mapping: HashMap<u16, ZoneName> = serde_yaml::from_reader(zone_file)
        .map_err(|err| anyhow!("Unable to read zone mapping file: {}", err))?;

    let dat_context = Arc::new(DatContext::from_path_and_zone_mappings(
        lookup_dir,
        zones_mapping,
    )?);

    let in_dir = project_path.join(RAW_DATA_DIR);
    let out_dir = project_path.join(DAT_GENERATION_DIR);
    let total_count = processor.all_yaml_to_dats(dat_context, &in_dir, &out_dir);
    println!("Generating {} DATs", total_count);

    let mut finished = 0;
    while finished < total_count {
        let msg = rx.recv()?;
        match msg.state {
            DatProcessingState::Working => {}
            DatProcessingState::Finished(_) => {
                finished += 1;
            }
            DatProcessingState::Error(err) => {
                return Err(anyhow!("Processing error: {}", err));
            }
        }
    }

    println!("Done");

    Ok(())
}
