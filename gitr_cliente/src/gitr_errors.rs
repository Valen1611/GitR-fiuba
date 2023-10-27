use std::error::Error;
use std::fmt;



#[derive(Debug)]
pub enum GitrError{
    //DirectoryCreationError,
    FileCreationError(String),
    FileWriteError(String),
    ObjectNotFound(String),
    FileReadError(String),

    //InvalidNumberOfArguments(usize, usize),
    //InvalidArguments(Vec<String>),

    NoHead,
    AlreadyInitialized,
    //NoRepository,
    InvalidArgumentError(String, String),
    LogError,
    CompressionError,
}

impl fmt::Display for GitrError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {

            //Self::DirectoryCreationError => write!(f, "ERROR: No se pudo crear el directorio."),
            Self::FileCreationError(path) => write!(f, "ERROR: No se pudo crear el archivo {}", path),
            Self::FileWriteError(path)=>write!(f, "ERROR: No se pudo escribir el archivo {}", path),
            Self::ObjectNotFound(obj) => write!(f, "ERROR: No se encontro el objeto {}", obj),
            Self::FileReadError(path) => write!(f, "No se pudo leer el archivo {}", path),
            Self::NoHead => write!(f, "ERROR: No se encontro HEAD"),
            Self::AlreadyInitialized => write!(f, "ERROR: El repositorio ya esta inicializado"),
           // Self::NoRepository => write!(f, "ERROR: No se encontro el repositorio"),
            Self::LogError => write!(f, "ERROR: No se pudo escribir en el log"),
            Self::CompressionError => write!(f, "No se pudo comprimir el archivo"),
            Self::InvalidArgumentError(got, usage) => write!(f, "Argumentos invalidos.\n    Recibi: {}\n    Uso: {}\n", got, usage),
    }
}
}



impl Error for GitrError {}