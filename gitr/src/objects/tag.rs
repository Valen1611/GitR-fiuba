// object <commit-sha1>
// type commit
// tag <tag-name>
// tagger <author-with-timestamp>

// <tag-message>

use chrono::Utc;

use crate::{command_utils::{get_user_mail_from_config, get_current_username, flate2compress, sha1hashing}, gitr_errors::GitrError};

#[derive(Debug)]

pub struct Tag{
    data: Vec<u8>,
    hash: String,
    tag_name: String,
    tag_message: String,
    commit_hash: String,
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
    pub fn new(tag_name: String, tag_message: String, commit_hash: String) -> Result<Self, GitrError>{
        let mut format_data = String::new();
        format_data.push_str(&format!("object {}\n", commit_hash));
        format_data.push_str("type commit\n");
        format_data.push_str(&format!("tag {}\n", tag_name));
        format_data.push_str(&format!("tagger {} <{}> {} -0300\n", get_current_username("cliente".to_string()), get_user_mail_from_config("cliente".to_string())?, Utc::now().timestamp()));
        format_data.push_str("\n");
        format_data.push_str(&format!("{}\n", tag_message));
        let size = format_data.as_bytes().len();
        let format_data_entera = format!("tag {}\0{}", size, format_data);
        let compressed_file = flate2compress(format_data_entera.clone())?;
        let hashed_file = sha1hashing(format_data_entera.clone());
        let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(Tag {data:compressed_file,hash: hashed_file_str, tag_name: tag_name, tag_message: tag_message,commit_hash: commit_hash })
    }
    

    pub fn save(&self,cliente: String) -> Result<(), GitrError>{
        crate::file_manager::write_object(self.data.clone(), self.hash.clone(),cliente)?;
        Ok(())
    }

    pub fn get_hash(&self) -> String{
        self.hash.clone()
    }
}


/*

cargo test --package gitr --lib -- tag::tests::test_tag_save --exact --color always
*/

//1700847004 -0300
#[cfg(test)]
mod tests{
    use crate::command_utils::create_annotated_tag;

    use super::*;
    
    #[test]
    fn test_tag_test_hash(){
                //si se va a correr este test, cambiar el timestamp a mano en new

        let tag_name = "nuevo".to_string();
        let tag_message = "mensajeeee".to_string();
        let commit_object = "5a80a3efc93f00c9143f0a7ed4888780a777e6e".to_string();
        let tag = Tag::new(tag_name, tag_message, commit_object).unwrap();
        let hash = tag.hash;
        let expected_hash = "9883f1bbaee5e89bdc8998cd532882619ca6d87e";

        assert_eq!(hash, expected_hash);
    }
    
    #[test]
    fn test_tag_save() {
        let tag_name = "nuevo".to_string();
        let tag_message = "mensajeeee".to_string();
        let commit_object = "5a80a3efc93f00c9143f0a7ed4888780a777e6e".to_string();
        //let tag = Tag::new(tag_name, tag_message, commit_object).unwrap();
        //tag.save("gianni".to_string()).unwrap();
        
        create_annotated_tag(tag_name, tag_message, "gianni".to_string()).unwrap();
    }
    

}