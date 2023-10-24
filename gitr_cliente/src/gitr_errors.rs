use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum GitrError{
    DirectoryCreationError,
    FileCreationError(String),
    FileWriteError(String),
    ObjectNotFound,
    FileReadError(String),
    NoHead,
    AlreadyInitialized,
    NoRepository,
}

impl fmt::Display for GitrError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DirectoryCreationError => write!(f, "ERROR: No se pudo crear el directorio."),
            Self::FileCreationError(path) => write!(f, "ERROR: No se pudo crear el archivo {}", path),
            Self::FileWriteError(path)=>write!(f, "ERROR: No se pudo escribir el archivo {}", path),
            Self::ObjectNotFound => write!(f, "ERROR: No se encontro el objeto"),
            Self::FileReadError(path) => write!(f, "ERROR: No se pudo leer el archivo {}", path),
            Self::NoHead => write!(f, "ERROR: No se encontro HEAD"),
            Self::AlreadyInitialized => write!(f, "ERROR: El repositorio ya esta inicializado"),
            Self::NoRepository => write!(f, "ERROR: No se encontro el repositorio"),
    }
}
}



impl Error for GitrError {}