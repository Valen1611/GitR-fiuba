use std::cmp::max;

pub struct Diff{
    pub lineas_eliminadas: Vec<(usize,String)>,
    pub lineas_agregadas: Vec<(usize,String)>,
    pub lineas: Vec<(usize,bool,String)>,
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
        lineas_eliminadas: vec![],
        lineas_agregadas: vec![],
        lineas: vec![],
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
    let mut stack = Vec::new(); //es un vector de Celdas (struct)

    let mut j = len_columna;
    let mut i = len_fila;

    let mut num_bloque_actual = matriz_coincidencias[j][i].valor;
    loop {
        //println!("i: {}, j: {}", i, j);
        if i == 0 && j == 0 {
            if matriz_coincidencias[j][i].es_match {
                stack.push(matriz_coincidencias[j][i].clone());
            }
            break;
        }

        if matriz_coincidencias[j][i].es_match {
            //println!("Pusheo un match");
            stack.push(matriz_coincidencias[j][i].clone());  
            num_bloque_actual -= 1;
            // me muevo a la diagonal
            if i != 0 {
                i -= 1;
            }
            if j != 0 {
                j -= 1;
            }
        } 
        else {
            if j == 0 {
                if matriz_coincidencias[j][i-1].valor == num_bloque_actual {
                    //me muevo a la diagonal
                    if i != 0 {
                        i -= 1;
                    } 
                    if j != 0 {
                        j -= 1;
                    }
                    continue;
                }
            }

            if i == 0 {
                if matriz_coincidencias[j-1][i].valor == num_bloque_actual {
                    //me muevo a la diagonal
                    if i != 0 {
                        i -= 1;
                    } 
                    if j != 0 {
                        j -= 1;
                    }
                    continue;
                }
            }

            if matriz_coincidencias[j-1][i-1].valor == num_bloque_actual {
                //me muevo a la diagonal
                if i != 0 {
                    i -= 1;
                } 
                if j != 0 {
                    j -= 1;
                }
                continue;
            }
            // voy todo para arriba a buscar la esquina
            else {
                let mut k = j;
                let mut la_encontre_yendo_arriba = false;

                loop {
                    if matriz_coincidencias[k][i].es_match {
                        // listo, aca tengo que pushear
                        la_encontre_yendo_arriba = true;
                        j = k;
                        
                        stack.push(matriz_coincidencias[j][i].clone());  
                        // me muevo a la diagonal
                        if i != 0 {
                            i -= 1;
                        }
                        if j != 0 {
                            j -= 1;
                        }
                        
                        num_bloque_actual -= 1;

                        break;
                    }

                    if matriz_coincidencias[k-1][i].valor != num_bloque_actual {
                        // no lo encontre, lo voy a buscar a la izq
                        
                        break;
                    }
               
                    if k != 0 {
                        k -= 1;
                    }
                }
            // si no la encontre yendo para arriba, la voy a buscar a la izquierda
                if !la_encontre_yendo_arriba {
                    let mut k = i;

                    loop {
                        if matriz_coincidencias[j][k].es_match {
                            // listo, aca tengo que pushear
                            
                            i = k;
                            
                            stack.push(matriz_coincidencias[j][i].clone());  
                            // me muevo a la diagonal
                            if i != 0 {
                                i -= 1;
                            }
                            if j != 0 {
                                j -= 1;
                            }
                            
                            num_bloque_actual -= 1;

                            break;
                        }

                        if matriz_coincidencias[j][k-1].valor != num_bloque_actual {
                            // no lo encontre, lo voy a buscar a la izq
                            
                            break;
                        }
                    
                        if k != 0 {
                            k -= 1;
                        }
                    }
                }
                
                
               
            }


        }
       

    }

    /*
    0 0 0 0 0 0 0 0 0 
    0 1 1 1 1 1 1 1 1 
    0 1 2 2 2 2 2 2 2 
    0 1 2 2 2 2 2 2 2 
    0 1 2 2 2 2 2 2 2 
    0 1 2 2 2 3 3 3 3 
    0 1 2 2 2 3 4 4 4 
    0 1 2 2 2 3 4 5 5 
    0 1 2 2 2 3 4 5 5 
    0 1 2 2 2 3 4 5 5 
     */
    // vieja

    // loop {
    //     println!("i: {}, j: {}", i, j);
    //     if i == 0 && j == 0 {
    //         if matriz_coincidencias[j][i].es_match {
    //             stack.push(matriz_coincidencias[j][i].clone());
    //         }
    //         break;
    //     }

    //     if matriz_coincidencias[j][i].es_match {
    //         println!("Pusheo un match");
    //         stack.push(matriz_coincidencias[j][i].clone());
    //          // me muevo a la diagonal
    //         if i != 0 {
    //             i -= 1;
    //         }
    //         if j != 0 {
    //             j -= 1;
    //         }

    //         continue;
    //     } else {
    //         //me muevo a la diagonal
    //         if i != 0 {
    //             i -= 1;
    //         } 
    //         if j != 0 {
    //             j -= 1;
    //         }
    //         //  else {
    //         //     // estoy en 0,0
    //         //     break;
    //         // }
    //         continue;
    //     }
    // }
    //println!("Stack filtro {:?}", stack);
    for value in stack.iter().rev() {
        //println!("{}: ({},{})", value.valor_matcheado, value.fila, value.columna);
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
        //println!("base_lines: {:?}", base_lines);
        //println!("new_lines: {:?}", new_lines);
        
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
        // print the matrix
        for (i, row) in matriz_coincidencias.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                //print!("{} ", cell.valor);
            }
            //println!();
        }




        let (indices_lineas_eliminadas, indices_lineas_agregadas) = get_diff(matriz_coincidencias, base_lines.len()-1, new_lines.len()-1);

        let mut lineas_eliminadas = Vec::new(); //las que tengo que sacar de base: push(i,false,base[i])
        let mut lineas_agregadas = Vec::new(); //las que tengo que agrega a base: push(i,true,new[i])

        let mut lineas = Vec::new();

        for (i, line) in base_lines.iter().enumerate() {
            if indices_lineas_eliminadas.contains(&i) {
                println!("{}. -{}",i, line);
                lineas.push((i, false, line.to_string()));
            }
        }
        for (i, line) in new_lines.iter().enumerate() {
            if indices_lineas_agregadas.contains(&i) {
                println!("{}. +{}",i, line);
                lineas.push((i, true, line.to_string()));
            }
        } 
        lineas.sort_by(|a, b| a.0.cmp(&b.0)); //ordeno ascendente
        //println!("lineas: {:?}", lineas);

        for (i, line) in base_lines.iter().enumerate() {
            if indices_lineas_eliminadas.contains(&i) {
                println!("{}. -{}",i, line);
                lineas_eliminadas.push((i, line.to_string()));
            }
        }
        for (i, line) in new_lines.iter().enumerate() {
            if indices_lineas_agregadas.contains(&i) {
                println!("{}. +{}",i, line);
                lineas_agregadas.push((i, line.to_string()));
            }
        }
        
        
        Diff{
            lineas_eliminadas: lineas_eliminadas,
            lineas_agregadas: lineas_agregadas,
            lineas: lineas,
        }
    }

    pub fn has_delete_diff(&self,i:usize)->bool{
        // for line in self.lineas_eliminadas.iter(){
        //     if line.0 == i{
        //         return true;
        //     }
        // }
        // false

        for line in self.lineas_eliminadas.iter(){
            if line.0 == i{
                return true;
            }
        }
        false
    }

    pub fn has_add_diff(&self,i:usize) -> (bool,String){
        // let linea = (false,"".to_string());
        // for line in self.lineas_agregadas.iter(){
        //     if line.0 == i{
        //         return (true,line.1.clone());
        //     }
        // }
        // linea
        println!("lineas agregadas: {:?}", self.lineas_agregadas);
        let linea: (bool, String) = (false,"".to_string());
        for line in self.lineas_agregadas.iter(){
            if line.0 == i {
                return (true,line.1.clone());
            }
        }
        linea
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
        assert!(diff.lineas_eliminadas.is_empty());
    }

    #[test]
    fn test01_diff_entre_strings_diferentes_no_esta_vacio(){
        let base = "A\nB\nC\nD\nE\nF\nK".to_string();
        let new = "B\nH\nD\nE\nF\nC\nK".to_string();


        // let base = format!("fn main () {{\tprintln!(\"hello word!\");}}\nkasjdklajsd");

        // let new = format!("fn main () {{\tprintln!(\"hello word!\");}}\nTEXti en el medio\nkasjdklajsd");

        let diff = Diff::new(base,new);
        //assert!(!diff.chunk.is_empty());
    }

    #[test]
    fn test02_diff_con_codigo_elimino_linea(){
        let base = format!("fn main () {{\tprintln!(\"hello word!\");}}\n");

        let new = format!("fn main () {{\tprintln!(\"hello word!\");}}\na");
        
        let diff = Diff::new(base,new);
    }
}