use crate::file_manager;
use std::error::Error;

pub struct Blob{
    data: String
}

impl Blob{
    pub fn new(data: String) -> Self {
        Blob { data }
    }
    pub fn save(&self) -> Result<(), Box<dyn Error>>{

        file_manager::write_object(self.data.clone())?;

        Ok(())
    }
}