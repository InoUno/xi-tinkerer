use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    sync::Arc,
};

use anyhow::{anyhow, Result};
use dats::{base::Dat, context::DatContext, dat_format::DatFormat};
use serde::Serialize;

use crate::dat_descriptor::DatUsage;

pub(crate) struct DatToYamlConverter {
    pub dat_context: Arc<DatContext>,
    pub raw_data_path: PathBuf,
}

impl DatUsage for DatToYamlConverter {
    fn use_dat<T: DatFormat + Serialize + for<'b> serde::Deserialize<'b>>(
        self,
        dat: Dat<T>,
    ) -> Result<()> {
        let data = self.dat_context.get_data_from_dat(&dat)?;

        fs::create_dir_all(&self.raw_data_path.parent().unwrap())?;
        let file = File::create(&self.raw_data_path).map_err(|err| {
            anyhow!(
                "Could not create at file {}: {}",
                self.raw_data_path.display(),
                err
            )
        })?;

        serde_yaml::to_writer(BufWriter::new(file), &data)?;

        Ok(())
    }
}

pub(crate) struct YamlToDatConverter {
    pub dat_context: Arc<DatContext>,
    pub raw_data_path: PathBuf,
    pub dat_root_path: PathBuf,
}

impl DatUsage for YamlToDatConverter {
    fn use_dat<T: DatFormat + Serialize + for<'a> serde::Deserialize<'a>>(
        self,
        dat: Dat<T>,
    ) -> Result<()> {
        let relative_dat_path = dat.get_relative_dat_path(&self.dat_context)?;
        let dat_path = self.dat_root_path.join(relative_dat_path);

        fs::create_dir_all(&dat_path.parent().unwrap())?;
        let mut dat_file = File::create(&dat_path)
            .map_err(|err| anyhow!("Could not create file at {}: {}", dat_path.display(), err))?;

        let raw_data_file = File::open(&self.raw_data_path).map_err(|err| {
            anyhow!(
                "Could open file at {}: {}",
                self.raw_data_path.display(),
                err
            )
        })?;
        let data: T = serde_yaml::from_reader(BufReader::new(raw_data_file))?;

        dat_file.write_all(&data.to_bytes()?)?;

        Ok(())
    }
}
