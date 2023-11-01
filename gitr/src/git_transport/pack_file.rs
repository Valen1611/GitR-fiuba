extern crate flate2;
use flate2::Decompress;
use crate::gitr_errors::{GitrError, self};
use crate::objects::git_object::GitObject;
use crate::objects::commit::Commit;
use crate::objects::tree::Tree;
use crate::objects::blob::Blob;
use crate::git_transport::ref_discovery::*;

//Info util:
//El packfile tiene varios chunks de datos, algunos codificados en UTF8 (leibles con rust), otros comprimidos con ZLib.
//El packfile trae un header con información en UTF8, luego hay miniheaders para cada objeto, la info comprimida y un checksum.
//Se me ocurre que se puede hacer una estructura que levante todo lo util del packfile y lo deje manipulable: (vector de objetos, con sus tipos y eso,
//que quede listo para poder crear un repo).
//Tambien se me ocurre que podemos usar esta misma estructura para inicializar un pack file para enviarlo por el socket.
//A priori: se reciben 4 bytes de la signature: Tiene que ser PACK sino tira error.
//Luego cae el numero de versión: Son 4 bytes
#[derive(Debug)]
pub struct PackFile{
    version: u32,
    pub objects: Vec<GitObject>,
}

fn decode(input: &[u8]) -> Result<(Vec<u8>,u64), std::io::Error> {
    let mut decoder = Decompress::new(true);
    let mut output:[u8; 1024] = [0;1024];
    decoder.decompress(input, &mut output, flate2::FlushDecompress::Finish)?;
    let cant_leidos = decoder.total_in();
    //println!("Input de tamaño: {} genera output de tamaño {}", cant_leidos, decoder.total_out());
    let output_return = output[..decoder.total_out() as usize].to_vec();
    
    Ok((output_return, cant_leidos))
}

fn parse_git_object(data: &[u8]) -> Result<(u8, usize, &[u8],usize), GitrError> {
    //println!("Entrada a parse_object {:?}",data);
    // Verifica si hay suficientes bytes para el encabezado mínimo
    if data.len() < 2 {
        return Err(GitrError::PackFileError("parse_git_object".to_string(),"No hay suficientes bytes para el encabezado mínimo".to_string()));
    }

    // Tipo del objeto (solo los primeros 3 bits)
    let object_type = (data[0] << 1 >> 5) & 0x07;
   
    // Longitud del objeto
    let mut length = (data[0]<<4>>4) as usize;
    let mut cursor = 1;
    let mut shift = 4;
    
    
    // Decodifica la longitud en formato de longitud variable
    while (data[cursor-1] & 0x80) != 0 {        
        length |= (data[cursor] as usize & 0x7F) << shift;
        cursor += 1;
        shift += 7;
        
        // Verifica si la longitud es demasiado grande
        if shift > 28 {
            return Err(GitrError::PackFileError("parse_git_object".to_string(),"La longitud es demasiado grande".to_string()));
        }
    }
    //print!("longitud del objeto descomprimido-{:#010b} - {}\n",length,length);
    //print!("cursor: {}\n",cursor);

    // Verifica si hay suficientes bytes para el contenido del objeto
    if data.len() < cursor + length {
        return Err(GitrError::PackFileError("parse_git_object".to_string(),"No hay suficientes bytes para el contenido del objeto".to_string()));
    }

    // Extrae el contenido del objeto
    let object_content = &data[cursor..];
    Ok((object_type, length, object_content, cursor))
}

fn create_commit_object(decoded_data: &[u8])->Result<GitObject,GitrError>{
    let data_str = String::from_utf8_lossy(decoded_data);
    let data_for_commit = data_str.split('\n').collect::<Vec<&str>>();
    let (mut parent, mut tree, mut author, mut committer, mut message) = ("None","None","None","None","None");

    for line in data_for_commit{
        if line.starts_with("tree"){
            tree = line.split(' ').collect::<Vec<&str>>()[1];
        }
        if line.starts_with("parent"){
            parent = line.split(' ').collect::<Vec<&str>>()[1];
        }
        if line.starts_with("author"){
            author = match line.split_once(' '){
                Some(tupla) => tupla.1,
                None => return Err(GitrError::PackFileError("create_commit_object".to_string(),"Error al parsear el author".to_string()))
            };
        }
        if line.starts_with("committer"){
            committer = match line.split_once(' '){
                Some(tupla) => tupla.1,
                None => return Err(GitrError::PackFileError("create_commit_object".to_string(),"Error al parsear el commiter".to_string()))
            };
        }
        if line.contains("\0") || line == ""{
            continue;
        }
        message = line;
    }
    
    let message_bien = "\n".to_owned()+message;

    let commit = GitObject::Commit(Commit::new_from_packfile(tree.to_string(), parent.to_string(), author.to_string(), committer.to_string(), message_bien.to_string()).unwrap());
    //println!("Commit creado: {:?}", commit);
    Ok(commit)
}

fn create_tree_object(decoded_data: &[u8])->Result<GitObject,GitrError>{
    let tree = GitObject::Tree(Tree::new_from_packfile(decoded_data)?);
    Ok(tree)
}

fn create_blob_object(decoded_data: &[u8])->Result<GitObject,GitrError>{
    let data_str = String::from_utf8_lossy(decoded_data);
    let blob = GitObject::Blob(Blob::new(data_str.to_string())?);

    Ok(blob)
}

fn git_valid_object_from_packfile(object_type: u8, decoded_data: &[u8])->Result<GitObject,GitrError>{
    let object = match  object_type{
        1 => create_commit_object(decoded_data)?,
        2 => create_tree_object(decoded_data)?,
        3 => create_blob_object(decoded_data)?,
        _ => return Err(GitrError::PackFileError("git_valid_object_from_packfile".to_string(),"Tipo de objeto no válido".to_string()))
    };
    Ok(object)
}

pub fn read_pack_file(buffer: &mut[u8]) -> Result<Vec<GitObject>, GitrError> {
    // Leemos el número de objetos contenidos en el archivo de pack
    let num_objects = match buffer[8..12].try_into(){
        Ok(vec) => vec,
        Err(_e) => return Err(gitr_errors::GitrError::PackFileError("read_pack_file".to_string(),"no se pudo obtener la # objetos".to_string()))
    };
    let num_objects = u32::from_be_bytes(num_objects);

    let mut objects = vec![];

    let mut index: usize = 0;
    for i in 0..num_objects {
        print!("=========index: {}, vuelta {}\n",index + 12, i);
        match parse_git_object(&buffer[12+index..]) {
            Ok((object_type, _length, object_content,cursor)) => {
                println!("Tipo del objeto: {}", object_type);
                //println!("Longitud del objeto: {}", length);
                let (decodeado, leidos) = decode(object_content).unwrap();
                //print!("leidos: {}\n",leidos);
                println!("Contenido del objeto: {:?}", String::from_utf8_lossy(&decodeado[..]));
                objects.push(git_valid_object_from_packfile(object_type, &decodeado[..])?);
                index += leidos as usize + cursor;
            }
            Err(err) => {
                println!("Error: {}", err);
                return Err(GitrError::PackFileError("read_pack_file".to_string(),"no se pudo parsear el objeto".to_string()));
            }
        }
    }
    println!("Sali del for, lei todos los objetos");
    Ok(objects)
}

impl PackFile{
    pub fn new_from_server_packfile(buffer: &mut[u8])->Result<PackFile, GitrError>{
        verify_header(&buffer[..=3])?;
        let version = extract_version(&buffer[4..=7])?;
        let objects = read_pack_file(buffer)?;

        Ok(PackFile {
            version: version,
            objects: objects,})
    }
}

#[cfg(test)]
mod tests{
    use std::{net::TcpStream, io::{Write, Read}};

    use super::*;

    #[test]
    fn test00_receiveing_wrong_signature_throws_error(){
        let mut buffer= [(13),(14),(23),(44)];
        assert!(PackFile::new_from_server_packfile(&mut buffer).is_err());
    }

    #[test]
    fn test01_connection_to_daemon_is_succesful(){
        let mut socket = TcpStream::connect("localhost:9418").unwrap();
        assert!(socket.write("003cgit-upload-pack /mi-repo\0host=localhost:9418\0\0version=1\0".as_bytes()).is_ok());
    }

    #[test]
    fn test02_reference_discovery_al_daemon_discovers_correctly(){
        let mut socket = TcpStream::connect("localhost:9418").unwrap();
        let _ =socket.write("003cgit-upload-pack /mi-repo\0host=localhost:9418\0\0version=1\0".as_bytes());
        println!("Envío git-upload-pack al daemon");

        let mut buffer = [0;1024];
        let mut _bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
        let bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
        
        let references = discover_references(String::from_utf8_lossy(&buffer[..bytes_read]).to_string()).unwrap();
        println!("References: {:?}", references);
    }
    
    #[test]
    fn test03_hago_el_want_y_decodeo_los_objetos(){
        //let references = discover_references(String::from_utf8_lossy(&buffer[..bytes_read]).to_string()).unwrap();
        //socket.write(assemble_want_message(&references).unwrap().as_bytes()).unwrap();
        let mut socket = TcpStream::connect("localhost:9418").unwrap();
        let _ =socket.write("003cgit-upload-pack /mi-repo\0host=localhost:9418\0\0version=1\0".as_bytes());
        println!("Envío git-upload-pack al daemon");

        let mut buffer = [0;1024];
        let mut _bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
        println!("Recibido: {}",String::from_utf8_lossy(&buffer));

        let mut bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
        println!("Recibido: {}",String::from_utf8_lossy(&buffer));

        
        let references = discover_references(String::from_utf8_lossy(&buffer[..bytes_read]).to_string()).unwrap();
        println!("References: {:?}", references);

        let want = assemble_want_message(&references).unwrap();
        println!("Mando el want: {:?}", want);
        socket.write(want.as_bytes()).unwrap();


        while bytes_read != 0{
            bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
            let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("String recibido: \n {}", received_data);
            if received_data == "0008NAK\n"{
                println!("corto por recibir NAK");
                break;
            }
            println!("Cantidad leida: {}",bytes_read);
        }
        
        let bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
        println!("Aca tendria que estar el packfile: {}",String::from_utf8_lossy(&buffer));
        let _packfile = PackFile::new_from_server_packfile(&mut buffer[..bytes_read]).unwrap();
        println!("Packfile: {:?}", _packfile);
    }

    #[test]
    fn test04_armo_un_packfile_con_lo_decodeado(){
        let mut socket = TcpStream::connect("localhost:9418").unwrap();
        let _ =socket.write("003cgit-upload-pack /mi-repo\0host=localhost:9418\0\0version=1\0".as_bytes());
        let mut buffer = [0;1024];
        let mut _bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
        let mut bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
        let references = discover_references(String::from_utf8_lossy(&buffer[..bytes_read]).to_string()).unwrap();
        let want = assemble_want_message(&references).unwrap();
        socket.write(want.as_bytes()).unwrap();
        loop{
            bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
            let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
            if received_data == "0008NAK\n"{
                println!("corto por recibir NAK");
                break;
            }
        }
        let bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
        println!("Aca tendria que estar el packfile: {}",String::from_utf8_lossy(&buffer));
        let _packfile = PackFile::new_from_server_packfile(&mut buffer[..bytes_read]).unwrap();
    }
}