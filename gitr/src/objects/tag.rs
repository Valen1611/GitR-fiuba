// object <commit-sha1>
// type commit
// tag <tag-name>
// tagger <author-with-timestamp>

// <tag-message>

use chrono::Utc;

use crate::{commands::command_utils::{get_user_mail_from_config, get_current_username, flate2compress, sha1hashing}, gitr_errors::GitrError};

#[derive(Debug)]

pub struct Tag{
    data: Vec<u8>,
    hash: String,
    commit_hash: String,
    //tag_name: String,
    //tag_message: String,
}

/*
tag <size>\0object <commit-sha1>\n
type commit\n
tag <tag-name>\n
tagger <author-with-timestamp>\n
\n
<tag-message>\n

*/

impl Tag{
    pub fn new(tag_name: String, tag_message: String, commit_hash: String, cliente: String) -> Result<Self, GitrError>{
        let mut format_data = String::new();
        format_data.push_str(&format!("object {}\n", commit_hash));
        format_data.push_str("type commit\n");
        format_data.push_str(&format!("tag {}\n", tag_name));
        format_data.push_str(&format!("tagger {} <{}> {} -0300\n", get_current_username(cliente.clone()), get_user_mail_from_config(cliente.clone())?, Utc::now().timestamp()));
        format_data.push_str('\n'.to_string().as_str());
        format_data.push_str(&format!("{}\n", tag_message));
        let size = format_data.as_bytes().len();
        let format_data_entera = format!("tag {}\0{}", size, format_data);
        let compressed_file = flate2compress(format_data_entera.clone())?;
        let hashed_file = sha1hashing(format_data_entera.clone());
        let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(Tag {data:compressed_file,hash: hashed_file_str, commit_hash: commit_hash/*tag_name: tag_name, tag_message: tag_message,commit_hash: commit_hash */})
    }
    
    pub fn new_tag_from_data(data: String) -> Result<Tag, GitrError>{
        let tag_elems = data.split("\0").collect::<Vec<&str>>();
        if tag_elems.len() != 2 || !tag_elems[0].contains("tag"){
         return Err(GitrError::InvalidTagError)
        }
        let tag_string = tag_elems[1].to_string();
        Ok(Self::new_tag_from_string(tag_string)?)
    }

    pub fn new_tag_from_string(data: String)->Result<Tag,GitrError>{
        let (mut name,mut message,mut commit ,mut tagger) = (String::new(), String::new(), String::new(), String::new());
        for line in data.lines() {
            let elems = line.split_once(" ").unwrap_or((line,""));
            match elems.0 {
                "object" => commit = elems.1.to_string(),
                "tag" => name = elems.1.to_string(),
                "tagger" => tagger = elems.1.to_string(),
                _ => message = line.to_string(),
            }
        }
        let tag = Tag::new_from_packfile(name, "\n".to_string()+&message,commit,tagger)?;
        Ok(tag)
    }

    pub fn new_from_packfile(tag_name: String, tag_message: String,commit: String ,tagger: String) -> Result<Tag,GitrError> {
        let mut format_data = String::new();
        format_data.push_str(&format!("object {}\n", commit));
        format_data.push_str("type commit\n");
        format_data.push_str(&format!("tag {}\n", tag_name));
        format_data.push_str(&format!("tagger {}",tagger));
        format_data.push_str("\n");
        format_data.push_str(&format!("{}\n", tag_message));
        let size = format_data.as_bytes().len();
        let format_data_entera = format!("tag {}\0{}", size, format_data);
        let compressed_file = flate2compress(format_data_entera.clone())?;
        let hashed_file = sha1hashing(format_data_entera.clone());
        let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(Tag {data:compressed_file,hash: hashed_file_str,commit_hash:commit})
    }

    pub fn save(&self,cliente: String) -> Result<(), GitrError>{
        crate::file_manager::write_object(self.data.clone(), self.hash.clone(),cliente)?;
        Ok(())
    }

    pub fn get_commit_hash(&self) -> String{
        self.commit_hash.clone()
    }

    pub fn get_hash(&self) -> String{
        self.hash.clone()
    }
    pub fn get_data(&self) -> Vec<u8>{
        self.data.clone()
    }
}




//1700847004 -0300
#[cfg(test)]
mod tests{
    use crate::commands::command_utils::create_annotated_tag;

    use super::*;
    
    #[test]
    fn test_tag_test_hash(){
                //si se va a correr este test, cambiar el timestamp a mano en new

        let tag_name = "nuevo".to_string();
        let tag_message = "mensajeeee".to_string();
        let commit_object = "5a80a3efc93f00c9143f0a7ed4888780a777e6e".to_string();
        let tag = Tag::new(tag_name, tag_message, commit_object, "Test".to_string()).unwrap();
        let hash = tag.hash;
        let expected_hash = "9883f1bbaee5e89bdc8998cd532882619ca6d87e";

        assert_eq!(hash, expected_hash);
    }
    
    #[test]
    fn test_tag_save() {
        let tag_name = "nuevo".to_string();
        let tag_message = "mensajeeee".to_string();
        //let commit_object = "5a80a3efc93f00c9143f0a7ed4888780a777e6e".to_string();
        //let tag = Tag::new(tag_name, tag_message, commit_object).unwrap();
        //tag.save("gianni".to_string()).unwrap();
        
        create_annotated_tag(tag_name, tag_message, "gianni".to_string()).unwrap();
    }
    

}
