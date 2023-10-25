extern crate flate2;
use flate2::Decompress;
//Info util:
//El packfile tiene varios chunks de datos, algunos codificados en UTF8 (leibles con rust), otros comprimidos con ZLib.
//El packfile trae un header con información en UTF8, luego hay miniheaders para cada objeto, la info comprimida y un checksum.
//Se me ocurre que se puede hacer una estructura que levante todo lo util del packfile y lo deje manipulable: (vector de objetos, con sus tipos y eso,
//que quede listo para poder crear un repo).
//Tambien se me ocurre que podemos usar esta misma estructura para inicializar un pack file para enviarlo por el socket.
//A priori: se reciben 4 bytes de la signature: Tiene que ser PACK sino tira error.
//Luego cae el numero de versión: Son 4 bytes
pub struct pack_file{

}

fn decode(input: &[u8]) -> Result<([u8;256],u64), std::io::Error> {
    let mut decoder = Decompress::new(true);
    let mut output:[u8; 256] = [0;256];
    decoder.decompress(input, &mut output, flate2::FlushDecompress::Finish)?;
    let cant_leidos = decoder.total_in();
    //println!("Input de tamaño: {} genera output de tamaño {}", cant_leidos, decoder.total_out());
    Ok((output, cant_leidos))
}

fn verify_header(header_slice: &[u8])->Result<(),String>{
    let str_received = String::from_utf8_lossy(header_slice);
    if (str_received != "PACK"){
        return Err("Signature incorrect: is not PACK".to_string());
    }
    Ok(())
}

fn extract_version(version_slice:&[u8])->Result<u32,String>{
    let version: [u8 ;4] = version_slice.try_into().unwrap_or([0;4]);
    if version == [0;4] {
        return Err("La versión no pudo extraerse".to_string())
    }
    let version = u32::from_be_bytes(version);
    println!("Versión del archivo de pack: {:?}", version);
    Ok(version)
}

fn parse_git_object(data: &[u8]) -> Result<(u8, usize, &[u8],usize), &'static str> {
    println!("Entrada a parse_object {:?}",data);
    // Verifica si hay suficientes bytes para el encabezado mínimo
    if data.len() < 2 {
        return Err("No hay suficientes bytes para el encabezado del objeto Git");
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
            return Err("Longitud de objeto Git no válida");
        }
    }
    print!("longitud del objeto descomprimido-{:#010b} - {}\n",length,length);
    print!("cursor: {}\n",cursor);

    // Verifica si hay suficientes bytes para el contenido del objeto
    if data.len() < cursor + length {
        return Err("No hay suficientes bytes para el contenido del objeto Git");
    }

    // Extrae el contenido del objeto
    let object_content = &data[cursor..];
    Ok((object_type, length, object_content, cursor))
}

pub fn read_pack_file(buffer: &mut[u8]) -> Result<(), String> {
    // Leemos el número de objetos contenidos en el archivo de pack
    let num_objects = buffer[8..12].try_into().unwrap_or([0;4]);
    if num_objects == [0;4] {
        return Err("La cantidad de objetos no pudo ser leída correctamente".to_string())
    }
    let num_objects = u32::from_be_bytes(num_objects);
    println!("Número de objetos en el archivo de pack: {:?}", num_objects);

    //let mut objects = vec![];

    let mut index: usize = 0;
    for i in 0..num_objects {
        print!("=========index: {}, vuelta {}\n",index + 12, i);
        match parse_git_object(&buffer[12+index..]) {
            Ok((object_type, length, object_content,cursor)) => {
                println!("Tipo del objeto: {}", object_type);
                println!("Longitud del objeto: {}", length);
                let (decodeado, leidos) = decode(object_content).unwrap();
                print!("leidos: {}\n",leidos);
                println!("Contenido del objeto: {:?}", String::from_utf8_lossy(&decodeado[..]));
                index += leidos as usize + cursor;
            }
            Err(err) => {
                println!("Error: {}", err);
                return Err("Error al parsear el objeto".to_string());
            }
        }
    }
    println!("Sali del for, lei todos los objetos");
    Ok(())
}

fn extract_head_hash(head_slice: &str)->String{
    let head_hash = head_slice.split(' ').collect::<Vec<&str>>()[0];
    head_hash.to_string().split_off(4)
}

fn extract_hash_and_ref(ref_slice: &str)->(String,String){
    let split = ref_slice.split(' ').collect::<Vec<&str>>();
    println!("Split: {:?}", split);
    let hash = split[0];
    let reference = split[1];
    (hash.to_string().split_off(4), reference.to_string())
}

pub fn discover_references(received_data: String) -> Result<Vec<(String,String)>,String>{
    let mut references: Vec<(String,String)> = vec![];
    let iter_refs: Vec<&str> = received_data.split('\n').collect();
    println!("{:?}", iter_refs);
    //Extraigo el primer hash al que apunta HEAD
    let head_hash = extract_head_hash(iter_refs[0]);
    println!("Llegué al discover ref {}",head_hash);
    references.push((head_hash,"HEAD".to_string()));
    
    for refs in &iter_refs[1..]{
        if *refs == ""{
            break;
        }
        references.push(extract_hash_and_ref(refs));
    }
    

    Ok(references)
}

impl pack_file{
    pub fn new_from(buffer: &mut[u8])->Result<pack_file, String>{
        verify_header(&buffer[..=3])?;
        let _version = extract_version(&buffer[4..=7])?;
        let _objects = read_pack_file(buffer);

        Ok(pack_file {  })
    }
}

#[cfg(test)]
mod tests{
    use std::{net::TcpStream, io::{Write, Read}};

    use super::*;

    fn show_lines_received(buffer: &[u8]){
        let received_data = String::from_utf8_lossy(&buffer);
        for line in received_data.split('\n'){
            println!("{}",line);
        }
    }

    #[test]
    fn test00_receiveing_wrong_signature_throws_error(){
        let mut buffer= [(13),(14),(23),(44)];
        assert!(pack_file::new_from(&mut buffer).is_err());
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
        let mut bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
        
        let references = discover_references(String::from_utf8_lossy(&buffer[..bytes_read]).to_string());
        println!("References: {:?}", references);   
    }
}