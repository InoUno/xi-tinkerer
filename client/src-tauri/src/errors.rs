use dats::base::DatError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    DatError(#[from] DatError),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

// we must manually implement serde::Serialize
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
