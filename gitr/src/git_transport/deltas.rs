/// en el pack-file vienen asi:
/// HEADER:
///     tipo de objeto (3 primeros bits)
///     offset respecto el objeto que deltifica (hay que restarlo a la pos actual)
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
///             # Estos no estan encodeados, se usan directamente, queda:
///                 * size = [s3 s2 s1] -> Cantidad de bits a copiar
///                 * offset = [o4 o3 o2 o1] -> Offset respecto el inicio del objeto base donde empezar a copiar
///             