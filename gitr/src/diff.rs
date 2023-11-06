use std::cmp::max;

pub struct Diff{
    pub indices_agregados: Vec<usize>,
    pub indices_eliminados: Vec<usize>,
}
#[derive(Clone)]
#[derive(Debug)]
struct Celda{
    valor: usize,
    es_match: bool,
    valor_matcheado: String,
    fila: usize,
    columna: usize,
}
// hash1 hash2
//--- file1
//+++ file2
//@@ -1,3 +1,3 @@
//Line1
//Line2
//-Line3 ->borrado en 2 pero aparece en 1
//+Line5 ->agregado en 2 pero no aparece en 1

fn empty_diff()->Diff{
    Diff{
        indices_agregados: vec![],
        indices_eliminados: vec![],
    }
}

fn valor_match(matriz: &Vec<Vec<Celda>>, i: usize,j:usize)->usize{
    if i == 0 || j == 0 || (i,j) == (0,0){
        return 1;
    }
    else{
        return matriz[i-1][j-1].valor + 1;
    }
}

fn valor_unmatch(matriz: &Vec<Vec<Celda>>, i: usize,j:usize)->usize{
    if i == 0 && j == 0{
        return 0;
    }
    else if i == 0{
        return matriz[i][j-1].valor;
    }
    else if j == 0{
        return matriz[i-1][j].valor;
    }
    else{
        return max(matriz[i-1][j].valor, matriz[i][j-1].valor);
    }
}

fn get_diff(matriz_coincidencias: Vec<Vec<Celda>>, len_columna: usize, len_fila: usize) -> (Vec<usize>, Vec<usize>){
    let mut stack = Vec::new();

    let mut j = len_columna;
    let mut i = len_fila;

    loop {
        println!("i: {}, j: {}", i, j);
        if i == 0 && j == 0 {
            if matriz_coincidencias[j][i].es_match {
                stack.push(matriz_coincidencias[j][i].clone());
            }
            break;
        }

        if matriz_coincidencias[j][i].es_match {
            stack.push(matriz_coincidencias[j][i].clone());
             // me muevo a la diagonal
            if i != 0 {
                i -= 1;
            }
            if j != 0 {
                j -= 1;
            }

            continue;
        } else {
            if i != 0 {
                i -= 1;
            } else if j != 0 {
                j -= 1;
            } else {
                // estoy en 0,0
                break;
            }
            continue;
        }
    }

    for value in stack.iter().rev() {
        println!("{}: ({},{})", value.valor_matcheado, value.fila, value.columna);
    }

    // podrian ser sets
    let base_numbers = stack.iter().map(|x| x.fila).collect::<Vec<usize>>();
    let new_numbers = stack.iter().map(|x| x.columna).collect::<Vec<usize>>();

    let mut lineas_eliminadas = Vec::new();
    for i in 0..(len_columna+1) {
        if !base_numbers.contains(&i) {
            lineas_eliminadas.push(i);
        }
    }

    let mut lineas_agregadas = Vec::new();
    for i in 0..(len_fila+1) {
        if !new_numbers.contains(&i) {
            lineas_agregadas.push(i);
        }
    }
   
    (lineas_eliminadas, lineas_agregadas)

}

impl Diff{
    pub fn new(base: String, new:String) -> Diff{
        
        if base == new {
            return empty_diff();
        }
        let base_lines = base.lines().collect::<Vec<&str>>();
        let new_lines = new.lines().collect::<Vec<&str>>();

        println!("base_lines: {:?}", base_lines);
        println!("new_lines:  {:?}", new_lines);
        
        let mut matriz_coincidencias: Vec<Vec<Celda>> = vec![vec![]];

        for (i, base_line) in base_lines.iter().enumerate(){
            for (j, new_line) in new_lines.iter().enumerate(){
                if base_line == new_line{ 
                    let next_value = valor_match(&matriz_coincidencias, i, j);
                    matriz_coincidencias[i].push(Celda{
                        valor:next_value, 
                        es_match:true,
                        valor_matcheado: base_line.to_string(),
                        fila:i,
                        columna:j});
                }
                else{
                    let next_value = valor_unmatch(&matriz_coincidencias, i, j);
                    matriz_coincidencias[i].push(
                        Celda {
                            valor: next_value, 
                            es_match: false,
                            valor_matcheado: "".to_string(),
                            fila: i, 
                            columna: j}
                        );
                }
            }
            matriz_coincidencias.push(vec![]);
        }

        let (lineas_eliminadas, lineas_agregadas) = get_diff(matriz_coincidencias, base_lines.len()-1, new_lines.len()-1);

        
        for (i, line) in base_lines.iter().enumerate() {
            if lineas_eliminadas.contains(&i) {
                println!("{}. -{}",i, line);
            }
        }
        for (i, line) in new_lines.iter().enumerate() {
            if lineas_agregadas.contains(&i) {
                println!("{}. +{}",i, line);
            }
        }
        
    
        Diff{
            indices_agregados: lineas_agregadas,
            indices_eliminados: lineas_eliminadas,
        }
    }
}

//merge branch2 en la branch1
//file->string base, string branch1, string branch2
//diff(base,branch1)
//diff(base,branch2)
//diferencias de diffs
//si hay conflictos, se resuelven
//aplicar los diffs que sobrevivan

#[cfg(test)]

mod tests{
    use super::*;

    #[test]
    fn test00_diff_entre_string_iguales_esta_vacio(){
        let base = "hola".to_string();
        let new = "hola".to_string();
        let diff = Diff::new(base,new);
        assert!(diff.indices_eliminados.is_empty());
    }

    #[test]
    fn test01_diff_entre_strings_diferentes_no_esta_vacio(){
        let base = "A\nB\nC\nD\nE\nF\nK".to_string();
        let new = "B\nH\nD\nE\nF\nC\nK".to_string();


        let base = format!("fn main () {{\tprintln!(\"hello word!\");}}\nkasjdklajsd");

        let new = format!("fn main () {{\tprintln!(\"hello word!\");}}\nTEXti en el medio\nkasjdklajsd");

        let diff = Diff::new(base,new);
        //assert!(!diff.chunk.is_empty());
    }

    #[test]
    fn test02_diff_con_codigo_elimino_linea(){
        let base = format!("fn main () {{\tprintln!(\"hello word!\");}}\n");

        let new = format!("fn main () {{\tprintln!(\"hello word!\");}}\na");
        
        
    }
}