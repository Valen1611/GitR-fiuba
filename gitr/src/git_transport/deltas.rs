use crate::gitr_errors::GitrError;

/// en el pack-file vienen asi:
/// HEADER:
///     * tipo de objeto (3 primeros bits)
///     * largo encodeado del objeto
///     * offset respecto el objeto que deltifica (hay que restarlo a la pos actual)
///         --- encodeado como el tamaño pero por cada byte que no sea el ultimo se le suma un 1 antes 
///             de hacer el corrimiento
/// CUERPO:
///     1) size del objecto base (encodeado)
///     2) size del objeto resultante (encodeado)
///     3) Instrucciones de reconstruccion:
///         - Nueva Data: [0xxxxxxx] [Data Nueva]... -> Primer bit = 0, resto = tamaño de la data nueva 
///         - Copiar de la base: [1abcdefg] -> Primer bit = 1, resto = cuales de los bytes de ofs y size se usan
///             
///                      [ofs 1] [ofs 2] [ofs 3] [ofs 4] [size 1] [size 2] [size 3]
/// 
///             (a = size 3) (b = size 2) (c = size 1) (d = ofs 4) (e = ofs 3) (f = ofs 2) (g = ofs 1)
///         
///             # Estos no estan encodeados, se usan directamente y si no se activan quedan en 0x00, queda:
///                 * offset = [o4 o3 o2 o1] -> Offset respecto el inicio del objeto base donde empezar a copiar
///                 * size = [s3 s2 s1] -> Cantidad de bits a copiar
///             

pub fn get_offset(data: &[u8]) -> Result<(usize,usize),GitrError> {
    let mut ofs: usize = 0;
    let mut cant_bytes: usize = 0;
    
    for byte in data {
        cant_bytes += 1;
        ofs = (ofs << 7 ) | (byte & 0x7f) as usize;
        if byte & 0x80 == 0 {
            break;
        }
        ofs += 1
    }
    Ok((ofs,cant_bytes))
}

fn parse_copy_instruction(instruction: Vec<u8>) -> Result<(usize,usize),GitrError> {
    if instruction.len() != 8{
        return Err(GitrError::PackFileError("parse_copy_instruction".to_string(), "Instruccion de copia invalida".to_string()));
    }
    let mut size: usize = 0;
    let mut ofs: usize = 0;
    let activator = instruction[0];
    let mut i: usize = 0;
    while i < 7 {
        if i < 3 {
            if (activator & (1 << i)) != 0 {
                size = (size << 8) | instruction[7-i] as usize;
            } else {
                size = (size << 8) | 0 as usize;
            }
        } else if i > 3 {
            if (activator & (1 << i)) != 0 {
                ofs = (ofs << 8) | instruction[7-i] as usize;
            } else {
                ofs = (ofs << 8) | 0 as usize;
            }
        }
        i += 1;
    }
    Ok((ofs,size))

}

pub fn transform_delta(data: &[u8], base: &[u8]) -> Result<Vec<u8>,GitrError>{
    let mut final_data: Vec<u8> = Vec::new();
    let mut i: usize = 0;
    while i < data.len() {
        let byte = data[i];
        if byte & 0x80 == 0 { // empieza con 0 -> nueva data
            let size = (byte <<1>>1) as usize;
            let new_data = &data[i+1..i+1+size];
            final_data.extend(new_data);
            i += size+1;
        } else { // empieza con 1 -> copiar de la base
            let (ofs, size) = parse_copy_instruction(data[i..i+8].to_vec())?;
            let base_data = &base[ofs..ofs+size];
            final_data.extend(base_data);
            i += 8;
        }
    }
    Ok(final_data)
}
