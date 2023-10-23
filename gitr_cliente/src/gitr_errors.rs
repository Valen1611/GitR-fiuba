use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum GitrError{
    DirectoryCreationError(String),
    FileCreationError(String),
    FileWriteError(String),
    ObjectNotFound(String),
    FileReadError(String),
    InvalidNumberOfArguments(usize, usize),
    InvalidArguments(Vec<String>),
}

impl fmt::Display for GitrError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DirectoryCreationError(path) => write!(f, "ERROR: No se pudo crear el directorio {}", path),
            Self::FileCreationError(path) => write!(f, "No se pudo crear el archivo {}", path),
            Self::FileWriteError(path) =>write!(f, "No se pudo escribir el archivo {}", path),
            Self::FileReadError(path) => write!(f, "No se pudo leer el archivo {}", path),
            Self::ObjectNotFound(obj) => write!(f, "No se encontro el objeto {}", obj),
            

            Self::InvalidNumberOfArguments(expected, got) => write!(f, "Invalid number of arguments. Expected {}, got {}", expected, got),
            Self::InvalidArguments(flags) => {
                let mut msg = String::from("Invalid arguments: ");
                for flag in flags {
                    msg.push_str(flag);
                    msg.push_str(" ");
                }
                write!(f, "{}", msg)
            }
    }
}
}



impl Error for GitrError {}