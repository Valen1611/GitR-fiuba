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
///             (g = ofs 1) (f = ofs 2) (e = ofs 3) (d = ofs 4) (c = size 1) (b = size 2) (a = size 3)
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

fn parse_copy_instruction(instruction: Vec<u8>) -> Result<(usize,usize,usize),GitrError> {
    let mut size: usize = 0;
    let mut ofs: usize = 0;
    let activator = instruction[0];
    let tamanio = (activator.count_ones() - 1) as usize;
    let mut i: usize = 0;
    let mut j: usize = 0;
    println!("activator: {:b}",activator);
    while i < 7 {
        if i < 3 {
            // println!("va al size, bit: {:08b}",(64>>i));
            if (activator & (64 >> i)) != 0 {
                size = (size << 8) | instruction[tamanio-j] as usize;
                j += 1;
            } else {
                size = (size << 8) | 0 as usize;
            }
        } else if i >= 3 {
            // println!("va al ofs, bit: {:08b}",(64>>i));
            if (activator & (64 >> i)) != 0 {
                ofs = (ofs << 8) | instruction[tamanio-j] as usize;
                j +=1;
            } else {
                ofs = (ofs << 8) | 0 as usize;
            }
        }
        i += 1;
    }
    Ok((ofs,size,j))

}

pub fn transform_delta(data: &[u8], base: &[u8]) -> Result<Vec<u8>,GitrError>{
    let mut final_data: Vec<u8> = Vec::new();
    let mut i: usize = 1;
    println!("base: {:?}",String::from_utf8_lossy(base));
    println!("data: {:?}",String::from_utf8_lossy(data));
    for b in base {
        if vec![*b] == ("\0".as_bytes()) {
            break;
        }
        i += 1;
    }
    let base = &base[i..];
    i = 0;
    while i < data.len() {
        let byte = data[i];
        if byte & 0x80 == 0 { // empieza con 0 -> nueva data
            let size = (byte <<1>>1) as usize;
            let new_data = &data[i+1..i+1+size];
            final_data.extend(new_data);
            i += size+1;
        } else { // empieza con 1 -> copiar de la base
            let (ofs, size,tamanio) = parse_copy_instruction(data[i..].to_vec())?;
            let base_data = &base[ofs..ofs+size];
            final_data.extend(base_data);
            i += 1+tamanio;
        }
    }
    Ok(final_data)
}
