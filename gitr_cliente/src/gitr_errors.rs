use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum GitrError{
    DirectoryCreationError,
    FileCreationError(String),
    FileWriteError(String),
}

impl fmt::Display for GitrError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DirectoryCreationError => write!(f, "ERROR: No se pudo crear el directorio."),
            Self::FileCreationError(path) => write!(f, "ERROR: No se pudo crear el archivo {}", path),
            Self::FileWriteError(path)=>write!(f, "ERROR: No se pudo escribir el archivo {}", path)
    }
}
}



impl Error for GitrError {}