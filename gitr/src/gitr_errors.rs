
use std::fmt;



#[derive(Debug)]
pub enum GitrError{
    //DirectoryCreationError,
    FileCreationError(String),
    FileWriteError(String),
    FileDeletionError(String),
    ObjectNotFound(String),
    FileReadError(String),
    FileDeleteError(String),
    //InvalidNumberOfArguments(usize, usize),
    //InvalidArguments(Vec<String>),
    NoCommitExisting(String),
    NoHead,
    AlreadyInitialized,
    NoRepository,
    InvalidArgumentError(String, String),
    LogError,
    CompressionError,
    TimeError,
    InvalidTreeError,
    ConnectionError,
    SocketError(String,String),
    PackFileError(String,String),
    BranchNonExistsError(String),
    BranchAlreadyExistsError(String),
}

impl fmt::Display for GitrError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BranchNonExistsError(branch) => write!(f, "error: branch '{}' not found.", branch),
            Self::FileDeletionError(fun) => write!(f, "En la funcion {} falló una eliminación", fun),
            //Self::DirectoryCreationError => write!(f, "ERROR: No se pudo crear el directorio."),
            Self::FileCreationError(path) => write!(f, "ERROR: No se pudo crear el archivo {}", path),
            Self::FileWriteError(path)=>write!(f, "ERROR: No se pudo escribir el archivo {}", path),
            Self::FileDeleteError(path) => write!(f, "ERROR: No se pudo borrar el archivo {}", path),
            Self::ObjectNotFound(obj) => write!(f, "ERROR: No se encontro el objeto {}", obj),
            Self::FileReadError(path) => write!(f, "No se pudo leer el archivo {}", path),
            Self::BranchAlreadyExistsError(branch) => write!(f, "error: a branch named '{}' already exists.", branch),
            Self::NoHead => write!(f, "ERROR: No se encontro HEAD"),
            Self::AlreadyInitialized => write!(f, "ERROR: El repositorio ya esta inicializado"),
            Self::NoRepository => write!(f, "ERROR: No se encontro el repositorio"),
            Self::NoCommitExisting(brch)=> write! (f, "fatal: Not a valid object name: '{}'", brch),
           // Self::NoRepository => write!(f, "ERROR: No se encontro el repositorio"),
            Self::LogError => write!(f, "ERROR: No se pudo escribir en el log"),
            Self::CompressionError => write!(f, "No se pudo comprimir el archivo"),
            Self::InvalidArgumentError(got, usage) => write!(f, "Argumentos invalidos.\n    Recibi: {}\n    Uso: {}\n", got, usage),
            Self::TimeError => write!(f, "No se pudo obtener el tiempo actual"),
            Self::InvalidTreeError => write!(f, "El arbol no es valido"),
            Self::ConnectionError => write!(f, "No se pudo conectar al servidor"),
            Self::SocketError(origin_function, info) => write!(f, "SocketError en la funcion {}. Info: {}", origin_function, info),
            Self::PackFileError(origin_function, info) => write!(f, "PackFileError en la funcion {}. Info: {}", origin_function, info),
    }
}

}

//impl Error for GitrError {}