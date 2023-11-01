use core::num;
use std::{fs::File, io::Read};
extern crate flate2;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
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

fn decode(input: &[u8]) -> Result<(Vec<u8>,usize), std::io::Error> {
    let mut decoder = ZlibDecoder::new(input);
    let traducidos = decoder.total_in();
    print!("traducidos: {}\n",traducidos);
    let mut decoded_data = Vec::new();
    let leidos = decoder.read_to_end(&mut decoded_data)?;
    Ok((decoded_data, leidos))
}

fn verify_header(header_slice: &[u8])->Result<(),String>{
    let str_received = String::from_utf8_lossy(header_slice);
    if (str_received != "PACK"){
        return Err("Signature incorrect: is not PACK".to_string());
    }
    Ok(())
}

fn extract_version(version_slice:&[u8])->Result<u32,String>{
    let mut version: [u8 ;4] = version_slice.try_into().unwrap_or([0;4]);
    if version == [0;4] {
        return Err("La versión no pudo extraerse".to_string())
    }
    let version = u32::from_be_bytes(version);
    println!("Versión del archivo de pack: {:?}", version);
    Ok(version)
}

fn parse_git_object(data: &[u8]) -> Result<(u8, usize, &[u8],usize), &'static str> {
    // println!("{:?}",data);
    // Verifica si hay suficientes bytes para el encabezado mínimo
    if data.len() < 2 {
        return Err("No hay suficientes bytes para el encabezado del objeto Git");
    }

    // Tipo del objeto (solo los primeros 3 bits)
    // print!("----->{:?}<------",data);
    let object_type = (data[0] << 1 >> 5) & 0x07;
    print!("data[0]:-{:#010b}\n",data[0]);
    print!("tipo de objeto: {:#010b} - {}\n",object_type,object_type);

    // Longitud del objeto
    let mut length = (data[0]<<4>>4) as usize;
    print!("len1-{:#010b} - {}\n",length,length);
    let mut cursor = 1;
    let mut shift = 4;

   
    // Decodifica la longitud en formato de longitud variable
    while (data[cursor-1] & 0x80) != 0 {
        // print!("entro al while\n");
        
        length |= (data[cursor] as usize & 0x7F) << shift;
        print!("data[{}]--{:#010b}\n",cursor,data[cursor]);
        print!("len1-{:#010b} - {}\n",length,length);
        cursor += 1;
        shift += 7;
        
        // Verifica si la longitud es demasiado grande
        if shift > 28 {
            return Err("Longitud de objeto Git no válida");
        }
    }
    print!("cursor:{:#010b}\n",data[cursor]);
    // length |= (data[cursor] as usize & 0x7F) << shift;
    // print!("len3-{}\n",length);
    // cursor += 1;

    // Verifica si hay suficientes bytes para el contenido del objeto
    if data.len() < cursor + length {
        return Err("No hay suficientes bytes para el contenido del objeto Git");
    }

    // Extrae el contenido del objeto
    length = length -75;
    let object_content = &data[cursor..cursor + length];
    print!("len obj:::{}\n",object_content.len());
    println!("CONTENIDO DEL OBJETO EN EL PARSER {:?}", object_content);
    Ok((object_type, length, object_content,cursor + length))
}

pub fn read_pack_file(buffer: &mut[u8]) -> Result<(), String> {
    // Leemos el número de objetos contenidos en el archivo de pack
    let mut num_objects = buffer[8..12].try_into().unwrap_or([0;4]);
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
            Ok((object_type, length, object_content,ind)) => {
                println!("Tipo del objeto: {}", object_type);
                println!("Longitud del objeto: {}", length);
                let (decodeado, leidos) = decode(object_content).unwrap();
                print!("leidos: {}\n",leidos);
                println!("Contenido del objeto: {:?}", String::from_utf8_lossy(&decodeado[..]));
                index += ind;
            }
            Err(err) => {
                println!("Error: {}", err);
                return Err("Error al parsear el objeto".to_string());
            }
        }
    }
    Ok(())
}


impl pack_file{
    pub fn new_from(buffer: &mut[u8])->Result<pack_file, String>{
        verify_header(&buffer[..=3])?;
        let version = extract_version(&buffer[4..=7])?;
        let objects = read_pack_file(buffer);

        Ok(pack_file {  })
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test00_receiveing_wrong_signature_throws_error(){
        let mut buffer= [(13),(14),(23),(44)];
        assert!(pack_file::new_from(&mut buffer).is_err());
    }
}

pub fn descomprimir_bruto(data: &[u8]) {
    let mut finall = "".to_string();
    for byte in data {
        match decode(data) {
            Ok((decoded_data, _leidos)) => {
                finall = finall + &String::from_utf8_lossy(&decoded_data[..]).to_string();
            }
            Err(err) => {
                finall = finall + &String::from_utf8_lossy(&[*byte]).to_string(); 
            }
            
        }
    }
    print!("[[[[[[[[[[[[finall: {}",finall)
}