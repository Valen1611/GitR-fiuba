use std::{io::{Write, Read}, fs::{self}, path::Path, collections::{HashMap, HashSet}, net::TcpStream};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Sha1, Digest};
use crate::{file_manager::{read_index, self, get_head, get_current_commit, get_current_repo, visit_dirs, update_working_directory}, diff::Diff};
use crate::{objects::{blob::{TreeEntry, Blob}, tree::Tree, commit::Commit}, gitr_errors::GitrError};


/***************************
 *************************** 
 *  DEFLATING AND HASHING
 **************************
 **************************/

/// compression function for Vec<u8>
pub fn flate2compress2(input: Vec<u8>) -> Result<Vec<u8>, GitrError>{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    match encoder.write_all(&input) {
        Ok(_) => {},
        Err(_) => return Err(GitrError::CompressionError),
    };
    let compressed_bytes = match encoder.finish() {
        Ok(bytes) => bytes,
        Err(_) => return Err(GitrError::CompressionError),
    };
    Ok(compressed_bytes)
}
/// hashing function for Vec<u8>
pub fn sha1hashing2(input: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(&input);
    let result = hasher.finalize();
    result.to_vec()
}
/// hashing function for String
pub fn sha1hashing(input: String) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.to_vec()
}
/// compression function for String
pub fn flate2compress(input: String) -> Result<Vec<u8>, GitrError>{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    match encoder.write_all(input.as_bytes()) {
        Ok(_) => {},
        Err(_) => return Err(GitrError::CompressionError),
    };
    let compressed_bytes = match encoder.finish() {
        Ok(bytes) => bytes,
        Err(_) => return Err(GitrError::CompressionError),
    };
    Ok(compressed_bytes)
}


/***************************
 *************************** 
 * CAT-FILE FUNCTIONS
 **************************
 **************************/


/// receives properties from an object and prints depending on the flag
pub fn print_cat_file_command(data_requested:&str, object_hash: &str, object_type:&str, res_output:String, size:&str)->Result<(),GitrError>{
    if data_requested == "-t"{
        println!("{}", object_type);
    }
    if data_requested == "-s"{
        println!("{}", size);
    }
    if data_requested == "-p"{
        let raw_data = match res_output.split_once('\0') {
            Some((_object_type, raw_data)) => raw_data,
            None => {
                println!("Error: invalid object type");
                return Err(GitrError::FileReadError(object_hash.to_string()))
            }
        };
        match object_type {
            "blob" => print_blob_data(raw_data),
            "tree" => print_tree_data(raw_data),
            "commit" => print_commit_data(raw_data),
            _ => println!("Error: invalid object type"),
        }
    }
    Ok(())
}
/// returns object hash, output, size and type
pub fn get_object_properties(flags:Vec<String>)->Result<(String, String, String, String), GitrError>{
    let object_hash = &flags[1];
    let res_output = file_manager::read_object(object_hash)?;
    let object_type = res_output.split(' ').collect::<Vec<&str>>()[0];
    let _size = res_output.split(' ').collect::<Vec<&str>>()[1];
    let size = _size.split('\0').collect::<Vec<&str>>()[0];
    Ok((object_hash.to_string(), res_output.clone(), size.to_string(), object_type.to_string()))
}


/***************************
 *************************** 
 *  OBJECT PRINTS
 **************************
 **************************/


pub fn print_blob_data(raw_data: &str) {
    println!("{}", raw_data);
}

pub fn print_tree_data(raw_data: &str) {
    let files = raw_data.split('\n').collect::<Vec<&str>>();
    for object in files {
        let file_atributes = object.split(' ').collect::<Vec<&str>>();
        let file_mode = file_atributes[0];
        let file_path_hash = file_atributes[1];
        let file_path = file_path_hash.split('\0').collect::<Vec<&str>>()[0];
        let file_hash = file_path_hash.split('\0').collect::<Vec<&str>>()[1];
        let file_type = if file_mode == "100644"{
            "blob"
        } else{
            "tree"
        };

        println!("{} {} {} {}", file_mode, file_type, file_hash, file_path);
    }
}
pub fn print_commit_data(raw_data: &str){
    println!("{}", raw_data);
}


/***************************
 *************************** 
 *  CHECKOUT FUNCTIONS
 **************************
 **************************/

/// create a tree (and blobs inside it) for checkout function
pub fn create_trees(tree_map:HashMap<String, Vec<String>>, current_dir: String) -> Result<Tree, GitrError> {
    let mut tree_entry: Vec<(String,TreeEntry)> = Vec::new();
    if let Some(objs) = tree_map.get(&current_dir) {
        for obj in objs {
                if tree_map.contains_key(obj) {
                    let new_tree = create_trees(tree_map.clone(), obj.to_string())?;
                    tree_entry.push((obj.clone(), TreeEntry::Tree(new_tree)));
            } else {
                let raw_data = file_manager::read_file(obj.clone())?;
                let blob = Blob::new(raw_data)?;
                tree_entry.push((obj.clone(), TreeEntry::Blob(blob)));
            }
        }
    };
    let tree = Tree::new(tree_entry)?;
    tree.save()?;
    Ok(tree)
}

/// writes the main tree for a commit, then writes the commit and the branch if necessary
pub fn get_tree_entries(message:String) -> Result<(), GitrError>{
    let (tree_map, tree_order) = get_hashmap_for_checkout()?;
    let final_tree = create_trees(tree_map, tree_order[0].clone())?;
    final_tree.save()?;
    write_new_commit_and_branch(final_tree, message)?;
    Ok(())
}
/// write a new commit and the branch if necessary
pub fn write_new_commit_and_branch(final_tree:Tree, message: String)->Result<(), GitrError>{
    let head = file_manager::get_head()?;
    let repo = file_manager::get_current_repo()?;
    let path_complete = repo.clone()+"/gitr/"+head.as_str();
    if fs::metadata(path_complete.clone()).is_err(){
        let dir = repo + "/gitr/refs/heads/master";
        file_manager::write_file(path_complete, final_tree.get_hash())?;
        if !Path::new(&dir).exists(){
            let current_commit = file_manager::get_current_commit()?;
            file_manager::write_file(dir.clone(), current_commit)?;
        }
        let commit = Commit::new(final_tree.get_hash(), "None".to_string(), get_current_username(), get_current_username(), message)?;
        commit.save()?;
        file_manager::write_file(dir, commit.get_hash())?;
    }else{
        let dir = repo + "/gitr/" + &head;
        let current_commit = file_manager::get_current_commit()?;
        let commit = Commit::new(final_tree.get_hash(), current_commit, get_current_username(), get_current_username(), message)?;
        commit.save()?;
        file_manager::write_file(dir, commit.get_hash())?;
    } 
    Ok(())
}

/// returns a hashmap to create trees (using the index)
pub fn get_hashmap_for_checkout()->Result<(HashMap<String, Vec<String>>,Vec<String>),GitrError>{
    let mut tree_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut tree_order: Vec<String> = Vec::new(); 
    let index_files = read_index()?;
    for file_info in index_files.split('\n') {
        ///// ojo aca
        let file_path = file_info.split(' ').collect::<Vec<&str>>()[3];
        let splitted_file_path = file_path.split('/').collect::<Vec<&str>>();
        println!("{}",file_path);
        for (i, dir) in (splitted_file_path.clone()).iter().enumerate() {
            if let Some(last_element) = splitted_file_path.last() {
                if dir == last_element {
                    update_hashmap_tree_entry(&mut tree_map, splitted_file_path[i-1], file_path.to_string());
                }else {
                        if !tree_map.contains_key(dir as &str) {
                            tree_map.insert(dir.to_string(), vec![]);
                            tree_order.push(dir.to_string());
                        }
                        if i == 0 {
                            continue;
                        }
                    update_hashmap_tree_entry(&mut tree_map, splitted_file_path[i-1], dir.to_string());
                }
            }
        }
    }
    Ok((tree_map, tree_order))
}

/// update the tree entries hashmap
pub fn update_hashmap_tree_entry(tree_map:&mut  HashMap<String, Vec<String>>, previous_dir: &str, file_path: String){
    if tree_map.contains_key(previous_dir) {
        match tree_map.get_mut(previous_dir) {
            Some(folder) => {
                if !folder.contains(&file_path.to_string()) {
                    folder.push(file_path.to_string());
                }  
            },
            None => {
                println!("No se encontro el folder");
            }
        }
    }
}

pub fn get_branch_to_checkout(args_received: Vec<String>) -> Result<String, GitrError>{
    let mut branch_to_checkout: String = args_received[0].clone();
    if args_received.len() == 2 && args_received[0] == "-b"{
        branch_to_checkout = args_received[1].clone();
        branch_newbranch_flag(branch_to_checkout.clone())?;
    }
    if !branch_exists(branch_to_checkout.clone()){
        return Err(GitrError::BranchNonExistsError(args_received[0].clone()));
    }
    Ok(branch_to_checkout)
}


/***************************
 *************************** 
 *    GET USER DATA
 **************************
 **************************/

/// returns the username
pub fn get_current_username() -> String{
    if let Some(username) = std::env::var_os("USER") {
        match username.to_str(){
            Some(username) => username.to_string(),
            None => String::from("User"),
        }
    } else{
        String::from("User")
    }
}
/// returns the mail from config
pub fn get_user_mail_from_config() -> Result<String, GitrError>{
    let config_data = match file_manager::read_file("gitrconfig".to_string()) {
        Ok(config_data) => config_data,
        Err(e) => {
            return Err(GitrError::FileReadError(e.to_string()))
        }
    };

    let lines = config_data.split('\n').collect::<Vec<&str>>();
    let email = lines[1].split('=').collect::<Vec<&str>>()[1].trim_start();
    Ok(email.to_string())
}



/***************************
 *************************** 
 *   BRANCH FUNCTIONS
 **************************
 **************************/

/// print all the branches in repo
pub fn print_branches()-> Result<(), GitrError>{
    let head = file_manager::get_head()?;
    let head_vec = head.split('/').collect::<Vec<&str>>();
    let head = head_vec[head_vec.len()-1];
    let branches = file_manager::get_branches()?;
        for branch in branches{
            if head == branch{
                let index_branch = format!("* \x1b[92m{}\x1b[0m", branch);
                println!("{}",index_branch);
                continue;
            }
            println!("{}", branch);
        }
    Ok(())
}

/// check if a branch exists
pub fn branch_exists(branch: String) -> bool{
    let branches = file_manager::get_branches();
    let branches = match branches{
        Ok(branches) => branches,
        Err(_) => return false,
    };
    for branch_name in branches{
        if branch_name == branch{
            return true;
        }
    }
    false
}

/// branch -d flag function
pub fn branch_delete_flag(branch:String)-> Result<(),GitrError>{
    if !branch_exists(branch.clone()){
        return Err(GitrError::BranchNonExistsError(branch))
    }
    file_manager::delete_branch(branch, false)?;
    Ok(())
}

/// branch -m flag function
pub fn branch_move_flag(branch_origin:String, branch_destination:String)->Result<(),GitrError>{
    if !branch_exists(branch_origin.clone()){
        return Err(GitrError::BranchNonExistsError(branch_origin))
    }
    if branch_exists(branch_destination.clone()){
        return Err(GitrError::BranchAlreadyExistsError(branch_destination))
    }
    let repo = get_current_repo()?;
    let old_path = format!("{}/gitr/refs/heads/{}", repo.clone(), branch_origin);
    let new_path = format!("{}/gitr/refs/heads/{}", repo.clone(), branch_destination);
    file_manager::move_branch(old_path.clone(), new_path.clone())?;
    let head = get_head()?;
    if branch_origin == head.split('/').collect::<Vec<&str>>()[2]{
        println!("{}", branch_origin);
        println!("{}", head.split('/').collect::<Vec<&str>>()[2]);
        let ref_correct = format!("refs/heads/{}", branch_destination);
        file_manager::update_head(&ref_correct)?;
    }   
    Ok(())
}

/// branch <newbranch> flag function
pub fn branch_newbranch_flag(branch:String) -> Result<(), GitrError>{
    let repo = get_current_repo()?;
    if branch_exists(branch.clone()){
        return Err(GitrError::BranchAlreadyExistsError(branch))
    }
    let current_commit = file_manager::get_current_commit()?;
    file_manager::write_file(format!("{}/gitr/refs/heads/{}", repo.clone(), branch), current_commit)?;
    Ok(())
    
}

pub fn branch_commits_list(branch_name: String)->Result<Vec<String>, GitrError>{
    let mut commits = Vec::new();
    let mut commit = file_manager::get_commit(branch_name)?;
    commits.push(commit.clone());
    loop {
        let parent = file_manager::get_parent_commit(commit.clone())?;

        if parent == "None" {
            break;
        }

        commit = parent;
        commits.push(commit.clone());
    }
    Ok(commits)
}
/***************************
 *************************** 
 *   COMMIT FUNCTIONS
 **************************
 **************************/

pub fn print_commit_confirmation(message:String)->Result<(), GitrError>{
    let branch = get_head()?
            .split('/')
            .collect::<Vec<&str>>()[2]
            .to_string();
        let hash_recortado = &get_current_commit()?[0..7];

        println!("[{} {}] {}", branch, hash_recortado, message);
        Ok(())
}
/// check if a commit exist
pub fn commit_existing() -> Result<(), GitrError>{
    let repo = file_manager::get_current_repo()?;
    let head = file_manager::get_head()?;
    let branch_name = head.split('/').collect::<Vec<&str>>()[2];
    if fs::metadata(repo.clone() + "/gitr/" + &head).is_err(){
        return Err(GitrError::NoCommitExisting(branch_name.to_string()))
    }
    Ok(())
}

/***************************
 *************************** 
 *   MERGE FUNCTIONS
 **************************
 **************************/

 
pub fn fast_forward_merge(branch_name:String)->Result<(),GitrError> {
    let commit: String = file_manager::get_commit(branch_name)?;
    let head = get_head()?;
    let repo = get_current_repo()?;
    let path = format!("{}/gitr/{}", repo, head);
    file_manager::write_file(path, commit.clone())?;
    update_working_directory(commit)?;
    Ok(())
}

pub fn get_blobs_from_commit(commit_hash: String)->Result<(),GitrError> {
    //entro al commit
    let path_and_hash_hashmap = get_commit_hashmap(commit_hash)?;
    
    println!("hashmap: {:?}", path_and_hash_hashmap);
    
    Ok(())
}

fn aplicar_difs(path: String, diff: Diff)-> Result<(), GitrError> {
    println!("=============PRINTS DE APLICAR_DIFFS=============");
    let string_archivo = file_manager::read_file(path.clone())?;
    let mut archivo_reconstruido = vec![];
    //println!("string_archivo: {:?}", string_archivo);
    //println!("diff a aplicar: {:?}", diff);
   // println!("diff:+ {:?}", diff.lineas_agregadas);
   // println!("diff:- {:?}", diff.lineas_eliminadas);




    let mut j = 0; //con este indexo el diff
    //let j = 0; //con este indexo el archivo
    let mut final_archivo = 0;
    for (i,line) in string_archivo.lines().enumerate(){
        if diff.lineas[j].0 == i{ //en la linea hay una operación
            if !diff.lineas[j].1{ //es un delete
                if j+1 == diff.lineas.len(){
                    break;
                }
                if diff.lineas[j+1].1{//hay un add tambien
                    archivo_reconstruido.push(diff.lineas[j+1].2.clone()+"\n"); //pusheo el add, ignorando lo que se borró
                    j+=2;
                }
                else{ //solo delete, no pusheo
                    j+=1;
                }
                continue
            }
            else{ //no hay delete, solo add
                archivo_reconstruido.push(line.to_string()+"\n"); //pusheo la linea del base
                archivo_reconstruido.push(diff.lineas[j].2.clone()+"\n"); //pusheo el add
            }
        }
        else{ //si no hay operacion, pusheo la linea del base y sigo
            archivo_reconstruido.push(line.to_string()+"\n");
        }
    }
    /*let mut lineas_archivo = string_archivo.lines().collect::<Vec<&str>>();
    loop{
        let linea_actual = lineas_archivo[j];
        if linea_actual.is_empty(){
            break;
        }
        if diff.lineas[i].0 == j{ //hay operaciones en esa linea
            if !diff.lineas[i].1{ //es un delete

            }
        }
    }*/
    /*for (i,line) in string_archivo.lines().enumerate(){
        let tiene_add = diff.has_add_diff(i);
        if diff.has_delete_diff(i){
            if tiene_add.0{
                print!(". Hay dif de add. Pusheo: {}",tiene_add.1.clone());
                archivo_reconstruido.push(tiene_add.1.clone()+"\n"); 
            }
            continue;
        }
        else if tiene_add.0 { 
            print!(". No hay dif de delete. Sí hay de add. Agrego: [{},{}]",line.to_string().clone(),tiene_add.1.clone());
            archivo_reconstruido.push(line.to_string()+"\n");
            archivo_reconstruido.push(tiene_add.1.clone()+"\n");
        } else {
            archivo_reconstruido.push(line.to_string()+"\n"); 
        }
    }*/

    
    for i in j..diff.lineas.len() { //agrego los diffs que me faltaron antes
        archivo_reconstruido.push(diff.lineas[j].2.clone()+"\n");
    }



    println!("archivo_reconstruido: {:?}", archivo_reconstruido);
    file_manager::write_file(path+"_mergeado", archivo_reconstruido.concat().to_string())?;
    println!("=======================================");
    println!("");
    Ok(())
}


fn comparar_diffs(diff_base_origin: Diff, diff_base_branch: Diff) -> Result<Diff, GitrError> {
    println!("=============PRINTS DE COMPARAR_DIFFS=============");

    let mut diff_final = Diff::new("".to_string(), "".to_string());
    
    let origin = diff_base_origin.lineas.clone();
    let new = diff_base_branch.lineas.clone();

    println!("diff_base_origin: {:?}", diff_base_origin.lineas);
    println!("diff_base_branch: {:?}", diff_base_branch.lineas);



    let mut joined_diffs = origin;
    joined_diffs.extend(new);
    let set: HashSet<_> = joined_diffs.clone().into_iter().collect(); 
    let mut result: Vec<_> = set.into_iter().collect();
    result.sort();


    let mut map: HashMap<usize, Vec<String>> = HashMap::new();
    for (index, flag, string) in result.clone() {
        if flag {
            map.entry(index).or_insert_with(Vec::new).push(string);
        }
     }
     /*
     RESULT:
        2.-"base"
        2.+"master"
        2.+"new"
        3.+"en new agrego una linea nueva"

     dic:
        2: ["master", "new"]
        3: ["en new agrego una linea nueva"]
      */



     // print map
    for (index, string) in map.clone() {
        println!("{}: {:?}", index, string);
    }

    let mut res_final = Vec::new();
    let mut conflict_abierto = false;

    let mut origin_conflicts: Vec<String> = Vec::new();
    let mut new_conflicts: Vec<String> = Vec::new();

    let mut indices_ya_visitados = HashSet::new();
    let mut indice_inicio_conflict = 0;

    for (index, flag, string) in result.clone() {
        //println!("index, flag, str: {} {} {}", index, flag, string);
        /*
        con algo que es asi
        2 lineas conflict
        1 linea ok en ambos
        2 lineas conflict

        funciona el diff pero se escribe todo doble.
    
        
         */

        
        
        if indices_ya_visitados.contains(&index) {
            continue;
        }
   

        if !flag {
            res_final.push((index, flag, string));
            continue;
        }

        let lineas = map.get(&index).unwrap(); //entra al diccionario y se trae una linea o varias si hay conflict
        if lineas.len() == 1 {
            res_final.push((index, flag, lineas[0].clone()));
            continue;
        }
        
        // si caigo aca es porque hay conflict
        
        if !indices_ya_visitados.contains(&(index-1)) {
            indice_inicio_conflict = index;
        }
        
        origin_conflicts.push(lineas[0].clone()+"\n");
        new_conflicts.push(lineas[1].clone()+"\n");
        indices_ya_visitados.insert(index);



        if indices_ya_visitados.contains(&(index-1)) //probablemente esto ande mal con 3 lineas de conflict
        {  

            let conflict = [
                "<<<<<<< HEAD\n",
                origin_conflicts.concat().as_str(),
                "========\n",
                new_conflicts.concat().as_str(),
                ">>>>>>> BRANCH"
                ].concat();
                
                res_final.push((indice_inicio_conflict, flag, conflict));
                
                origin_conflicts = Vec::new();
                new_conflicts = Vec::new();
        }

    }


    res_final.sort();
    diff_final.lineas = res_final.clone();    


    println!("**********************************");
    println!("* diff final: {:?}", diff_final.lineas);
    // for (i, accion, linea) in diff_final.lineas.clone(){
    //     println!("* linea: {}.{}{:?}", i, if accion {"+"} else {"-"}, linea);
    // }        
    println!("**********************************");

    println!("=======================================");
    println!("");
    Ok(diff_final)
}


fn comparar_diff_viejo(diff_base_origin: Diff, diff_base_branch: Diff) -> Result<Diff, GitrError> {
    println!("=============PRINTS DE COMPARAR_DIFFS=============");
    let (mut i, mut j) = (0,0);
    let mut diff_final = Diff::new("".to_string(), "".to_string());
    
    let origin = diff_base_origin.lineas.clone();
    let new = diff_base_branch.lineas.clone();
    
    println!("diff_base_origin: {:?}", diff_base_origin.lineas);
    println!("diff_base_branch: {:?}", diff_base_branch.lineas);

    // BASE
    // 0 hola
    // 1 soy
    // 2 pepe
    // 3 base 

    // ORIGIN
    //2. - base
    //2. + master

    // NEW
    //2. - base
    //2. + pepe 
    //3. + en new agrego linea

    let max_i = origin.len() - 1;
    let max_j = new.len() - 1;
    println!("max_i: {}", max_i);
    println!("max_j: {}", max_j);
    let conflict_abierto = false;
   // println!("max_i: {}", max_i);
    //println!("max_j: {}", max_j);
    println!("+++++ BEGIN LOOP +++++");
    loop {
        println!("i: {}", i);
        println!("j: {}", j);
        if i > max_i && j > max_j { //condicion de corte
             break;
        }

        if origin[i] == new[j] {
            // eliminan o agregan la misma linea, pusheo 
            diff_final.lineas.push(origin[i].clone());
            if i <= max_i {
                i+=1;
            }
            if j <= max_j {
                j+=1;
            }
            continue;
        }


        if origin[i].0 == new[j].0 && // ambas operan la misma linea
            origin[i].1 && new[j].1 { // ambos son de agregar
                // si caigo aca es porque agregan sobre la misma linea
                // cosas distintas, aca hay CONFLICT

                let pos_original = origin[i].0;
                let mut origin_conflicts = Vec::new();
                let mut new_conflicts = Vec::new();


                origin_conflicts.push(origin[i].2.clone()+"\n");
                new_conflicts.push(new[j].2.clone()+"\n");


                let conflict = [
                ">>>>>>>\n",
                origin_conflicts.concat().as_str(),
                "========\n",
                new_conflicts.concat().as_str(),
                "<<<<<<<"
                ].concat();

                println!("conflict: {:?}", conflict);

                diff_final.lineas.push((pos_original, true, conflict.clone()));

                // if conflict_abierto {
                //     // aca es porque ya existe un conflict que tiene
                //     // lineas guardadas
                //     continue;
                // }// y como lo cierro??

                if i <= max_i {
                    i+=1;
                }
                if j <= max_j {
                    j+=1;
                }
                continue;

        }





        // if i == max_i {
        //     println!("i llego al max");
        //     // si llegue aca es porque ya no hay mas lineas en origin
        //     // entonces pusheo las lineas que quedan en new
        //     diff_final.lineas.push(new[j].clone());
        //     if j != max_j {
        //         j+=1;
        //     }
        //     continue;
        // }
        // if j == max_j {
        //     println!("j llego al max");
        //     // si llegue aca es porque ya no hay mas lineas en new
        //     // entonces pusheo las lineas que quedan en origin
        //     diff_final.lineas.push(origin[i].clone());
        //     if i != max_i {
        //         i+=1;
        //     }
        //     continue;
        // }
    }

    println!("+++++ END LOOP +++++");

    // loop {
    //    // println!("i: {}", i);
    //    // println!("j: {}", j);
    //     if i >= max_i && j >= max_j { //condicion de corte
    //         break;
    //     }

    //     let indice_origin = origin[i].0;
    //     let indice_new = new[j].0;

    //     if origin[i] == new[i] {
    //         println!("iguales");
    //         println!("origin[i]: {:?}", origin[i]);
    //         println!("new[i]: {:?}", new[i]);
    //     }

    //     if indice_origin == indice_new{ //ambos diffs operan la misma linea (no se si es add o delete)
    //         if !origin[i].1 { //si en la linea origin borra
    //             // en este caso no es conflict, simplemente borro esa linea del original
    //             diff_final.lineas.push(origin[i].clone());
    //             //diff_final.lineas_eliminadas.push((origin[i].0, origin[i].2.clone()));
                
    //             if i != max_i {
    //                 i+=1;
    //             }
                
    //             if j != max_j {
    //                 j+=1;
    //             }
    //             continue;
    //         }
            
    //         let pos_original = origin[i].0;
    //         let mut origin_conflicts = Vec::new();
            
    //         origin_conflicts.push(origin[i].2.clone()+"\n");
            
    //         let mut k = i;
    //         //println!("i");
    //         loop {
    //             //println!("k: {}", k);
    //             //println!("origin_conflicts: {:?}", origin_conflicts);
    //             if k >= max_i{
    //                 break;
    //             }
    //             if max_i - 2 > k { 
    //                 if origin[k].0 + 1 != origin[k+1].0 { 
    //                     break;
    //                 }
    //                 if origin[k].0 + 1 == origin[k+1].0 && !origin[k+1].1 && origin[k].0 + 1 == origin[k+2].0 && origin[k+2].1 {
    //                     origin_conflicts.push(origin[k+2].2.clone()+"\n");
    //                 }
    //             }

    //             if k != max_i {
    //                 k+=1;
    //             }
    //         }

    //         if k != max_i {
    //             i=k;
    //         }
    //         else {
    //             i=max_i-1;
    //         }

    //         //println!("j");
    //         let mut new_conflicts = Vec::new();
    //         new_conflicts.push(new[j].2.clone()+"\n");
    //         let mut k = j;
    //         loop {
    //             //println!("k: {}", k);
    //             //println!("new_conflicts: {:?}", new_conflicts);
    //             if k >= max_j{
    //                 break;
    //             }
    //             if max_j - 2 > k { 
    //                 if new[k].0 + 1 != new[k+1].0 { 
    //                     break;
    //                 }
                    
    //                 if new[k].0 + 1 == new[k+1].0 && !new[k+1].1 && new[k].0 + 1 == new[k+2].0 && new[k+2].1 {
    //                     new_conflicts.push(new[k+2].2.clone()+"\n");
                        
    //                 }
    //             }

    //             if k < max_j {
    //                 k+=1;
    //             }
    //         }
    //         //println!("k: {}", k);
            
    //         if k != max_j {
    //             j=k;
    //         }
    //         else {
    //             j=max_j-1;
    //         }

    //         if i < max_i {
    //             i+=1;
    //         }
            
    //         if j < max_j {
    //             j+=1;
    //         }

    //         let conflict = [">>>>>>>\n",
    //             origin_conflicts.concat().as_str(),
    //             "========\n",
    //             new_conflicts.concat().as_str(),
               
    //             "<<<<<<<"].concat();

    //         println!("conflict: {:?}", conflict);

    //         diff_final.lineas.push((pos_original,true,conflict.clone()));
    //         diff_final.lineas_agregadas.push((pos_original, conflict.clone()));

    //         continue;


    //     }
    //     else if origin[i].0 < new[j].0{
    //         diff_final.lineas.push(origin[i].clone());
    //         if !origin[i].1 {
    //             diff_final.lineas_eliminadas.push((origin[i].0, origin[i].2.clone()));
    //         } else {
    //             diff_final.lineas_agregadas.push((origin[i].0, origin[i].2.clone()));
    //         }
    //         if i < max_i {
    //             i+=1;
    //         }
    //     }
    //     else{
    //         diff_final.lineas.push(new[j].clone());
    //         if !new[j].1 {
    //             diff_final.lineas_eliminadas.push((new[j].0, new[j].2.clone()));
    //         } else {
    //             diff_final.lineas_agregadas.push((new[j].0, new[j].2.clone()));
    //         }
    //         if j < max_j {
    //             j+=1;
    //         }
    //     }
       
    // }
    // print diff final
    println!("**********************************");
    println!("* diff final:");
    for (i, accion, linea) in diff_final.lineas.clone(){
        println!("* linea: {}.{}{:?}", i, if accion {"+"} else {"-"}, linea);
    }        
    println!("**********************************");


    println!("====================================================");
    println!("");
    Ok(diff_final)
}


pub fn three_way_merge(base_commit: String, origin_commit: String, branch_commit: String) -> Result<(), GitrError> {
    let branch_hashmap = get_commit_hashmap(branch_commit.clone())?;
    let mut origin_hashmap = get_commit_hashmap(origin_commit.clone())?;
    file_manager::add_new_files_from_merge(origin_hashmap.clone(), branch_hashmap.clone())?;
    origin_hashmap = get_commit_hashmap(origin_commit.clone())?;
    let base_hashmap = get_commit_hashmap(base_commit.clone())?;
    for (path, origin_file_hash) in origin_hashmap.iter(){

        let origin_file_data =file_manager::read_file(path.clone())?; 
        
        
        if branch_hashmap.contains_key(&path.clone()){
            let branch_file_hash = branch_hashmap[path].clone(); //aax
            let branch_file_data = file_manager::read_file_data_from_blob_hash(branch_file_hash.clone())?;

            
            if origin_file_hash == &branch_file_hash{
                continue;
            }
            
            let base_file_hash = base_hashmap[path].clone(); // chequear que capaz puede no exisiir en base
            let base_file_data = file_manager::read_file_data_from_blob_hash(base_file_hash.clone())?;

            if &base_file_hash == origin_file_hash {
                let diff_base_branch = Diff::new(base_file_data, branch_file_data);
                aplicar_difs(path.clone(), diff_base_branch)?;
                continue;
            }
            
            if base_file_hash == branch_file_hash {              
                continue;
           
            }
    
            println!("base_file_data: {:?}", base_file_data);
            println!("origin_file_data: {:?}", origin_file_data);
            println!("branch_file_data: {:?}", branch_file_data);

            let diff_base_origin = Diff::new(base_file_data.clone(), origin_file_data.clone());
            let diff_base_branch = Diff::new(base_file_data, branch_file_data);
            let union_diffs = comparar_diffs(diff_base_origin, diff_base_branch)?; //une los diffs o da el conflict
            //println!("union_diffs: {:?}", union_diffs);
            aplicar_difs(path.clone(), union_diffs)?;
        }
        else{
            continue;
        }
    }
    Ok(())
}


/***************************
 *************************** 
 *   STATUS FUNCTIONS
 **************************
 **************************/

 pub fn get_working_dir_hashmap() -> Result<HashMap<String, String>, GitrError>{
    let mut working_dir_hashmap = HashMap::new();
    let repo = file_manager::get_current_repo()?;
    let path = Path::new(repo.as_str());
    let files= visit_dirs(path);
    for file_path in files {
        let file_data = file_manager::read_file(file_path.clone())?;
        
        let blob = Blob::new(file_data.clone())?;
        let hash = blob.get_hash();
        working_dir_hashmap.insert(file_path, hash);
    }
    Ok(working_dir_hashmap)
}

pub fn status_print_to_be_comited(to_be_commited: &Vec<String>)->Result<(), GitrError>{
    let working_dir_hashmap = get_working_dir_hashmap()?;
    if !to_be_commited.is_empty() {
        println!("Changes to be committed:");
        println!("  (use \"rm <file>...\" to unstage)");
        for file in to_be_commited.clone() {
            let file_name = match file.split_once ('/'){
                Some((_path, file)) => file.to_string(),
                None => file.to_string(),
            };
            if !working_dir_hashmap.contains_key(file.as_str()) {
                println!("\t\x1b[31mdeleted:   {}\x1b[0m", file_name);
            }else{
                println!("\t\x1b[92mmodified   {}\x1b[0m", file_name);
            }
        }
    }
    Ok(())
}

pub fn status_print_not_staged(not_staged: &Vec<String>) {
    if !not_staged.is_empty() {
        println!("Changes not staged for commit:");
        println!("  (use \"add <file>...\" to update what will be committed)");
        println!("  (use \"rm <file>...\" to discard changes in working directory)");

        for file in not_staged.clone() {
            let file_name = match file.split_once ('/'){
                Some((_path, file)) => file.to_string(),
                None => file,
            };
            println!("\t\x1b[31mmodified:   {}\x1b[0m", file_name);
        }
    }
}

pub fn status_print_untracked(untracked_files: &Vec<String>, hayindex: bool) {
    if !untracked_files.is_empty() {
        println!("Untracked files:");
        println!("  (use \"add <file>...\" to include in what will be committed)");

        for file in untracked_files.clone() {
            let file_name = match file.split_once ('/'){
                Some((_path, file)) => file.to_string(),
                None => file,
            };
            println!("\t\x1b[31m{}\x1b[0m", file_name);
        }

        if !hayindex {
            println!("nothing added to commit but untracked files present (use \"add\" to track)");
        }
    }
}


pub fn status_print_current_branch() -> Result<(), GitrError> {
    let head = file_manager::get_head()?;
    let current_branch = head.split('/').collect::<Vec<&str>>()[2];
    println!("On branch {}", current_branch);
    if commit_existing().is_err(){
        println!("No commits yet");
    }
    Ok(())
}


pub fn get_index_hashmap() -> Result<(HashMap<String, String>, bool), GitrError> {
    let mut index_hashmap = HashMap::new();
    let mut hayindex = true;
    let index_data = match file_manager::read_index() {
        Ok(data) => data,
        Err(_) => {
            hayindex = false;
            String::new()
        }
    };
    if hayindex {
        for index_entry in index_data.split('\n') {
            let attributes = index_entry.split(' ').collect::<Vec<&str>>();
            let path = attributes[3].to_string();
            let hash = attributes[1].to_string();
            index_hashmap.insert(path, hash);
        }
    }
    Ok((index_hashmap, hayindex))
}

pub fn get_subtrees_data(hash_of_tree_to_read: String, file_path: String, tree_hashmap: &mut HashMap<String, String>) -> Result<(), GitrError>{
    let tree_data = file_manager::read_object(&hash_of_tree_to_read)?;

    let tree_entries = match tree_data.split_once('\0') {
        Some((_tree_type, tree_entries)) => tree_entries,
        None => "",
    };
    for entry in tree_entries.split('\n') {
        if entry.split(' ').collect::<Vec<&str>>()[0] == "40000"{
            let attributes = entry.split(' ').collect::<Vec<&str>>()[1];
            let relative_file_path= attributes.split('\0').collect::<Vec<&str>>()[0].to_string();
            let file_path = format!("{}/{}", file_path, relative_file_path);
            let file_hash = attributes.split('\0').collect::<Vec<&str>>()[1].to_string();
            get_subtrees_data(file_hash, file_path, tree_hashmap)?;
        }
        if entry.split(' ').collect::<Vec<&str>>()[0] == "40000"{
            continue;
        }


        let attributes = entry.split(' ').collect::<Vec<&str>>()[1];
        let relative_file_path= attributes.split('\0').collect::<Vec<&str>>()[0].to_string();
        let file_path = format!("{}/{}", file_path, relative_file_path);
        let file_hash = attributes.split('\0').collect::<Vec<&str>>()[1].to_string();

        tree_hashmap.insert(file_path, file_hash);
    };
    Ok(())
}


pub fn get_commit_hashmap(commit: String) -> Result<HashMap<String, String>, GitrError> {
      let mut tree_hashmap = HashMap::new();
    let current_commit = get_current_commit()?;
    if current_commit == commit{
        let (index_hashmap, _) = get_index_hashmap()?;
        return Ok(index_hashmap);
    }
      if !commit.is_empty() {
        
        let repo = file_manager::get_current_repo()?;
        let tree = file_manager::get_main_tree(commit)?;
        let tree_data = file_manager::read_object(&tree)?;
        let tree_entries = match tree_data.split_once('\0') {
            Some((_tree_type, tree_entries)) => tree_entries,
            None => "",
        };          
        for entry in tree_entries.split('\n') {
            if entry.split(' ').collect::<Vec<&str>>()[0] == "40000"{
                let attributes = entry.split(' ').collect::<Vec<&str>>()[1];
                let _file_path= attributes.split('\0').collect::<Vec<&str>>()[0].to_string();
                let file_path = format!("{}/{}", repo, _file_path);
                let file_hash = attributes.split('\0').collect::<Vec<&str>>()[1].to_string();
                get_subtrees_data(file_hash, file_path, &mut tree_hashmap)?;
            }
            
            if entry.split(' ').collect::<Vec<&str>>()[0] == "40000"{
                continue;
            }

            let attributes = entry.split(' ').collect::<Vec<&str>>()[1];
            let _file_path= attributes.split('\0').collect::<Vec<&str>>()[0].to_string();
            let file_path = format!("{}/{}", repo, _file_path);
            let file_hash = attributes.split('\0').collect::<Vec<&str>>()[1].to_string();

            tree_hashmap.insert(file_path, file_hash);
        }

      }

      Ok(tree_hashmap)
}



pub fn get_untracked_notstaged_files()->Result<(Vec<String>, Vec<String>, bool), GitrError>{
    let working_dir_hashmap = get_working_dir_hashmap()?;
    let (index_hashmap, hayindex) = get_index_hashmap()?;
    let current_commit_hashmap = get_current_commit_hashmap()?;
    let mut not_staged = Vec::new();
    let mut untracked_files = Vec::new();
    for (path, hash) in working_dir_hashmap.clone().into_iter() {
        if !index_hashmap.contains_key(path.as_str()) && !current_commit_hashmap.contains_key(path.as_str()) {
            untracked_files.push(path.clone());
        }
        if current_commit_hashmap.contains_key(path.clone().as_str()){
            if let Some(commit_hash) = current_commit_hashmap.get(path.as_str()) {
                if &hash != commit_hash && !index_hashmap.contains_key(&path) {
                    not_staged.push(path.clone( ));
                }
            };
        }
        if index_hashmap.contains_key(path.as_str()){
            if let Some(index_hash) = index_hashmap.get(path.as_str()) {
                if &hash != index_hash {
                    not_staged.push(path);
                }
            };
        }
    }
    Ok((not_staged, untracked_files, hayindex))
}


pub fn get_current_commit_hashmap() -> Result<HashMap<String, String>, GitrError> {
      let mut tree_hashmap = HashMap::new();
      let mut haycommitshechos = true;
      let current_commit = match file_manager::get_current_commit() {
          Ok(commit) => commit,
          Err(_) => {
              haycommitshechos = false;
              String::new()
          }
      };
      
      if haycommitshechos {
        let repo = file_manager::get_current_repo()?;
        let tree = file_manager::get_main_tree(current_commit)?;
        let tree_data = file_manager::read_object(&tree)?;
        let tree_entries = match tree_data.split_once('\0') {
            Some((_tree_type, tree_entries)) => tree_entries,
            None => "",
        };
        for entry in tree_entries.split('\n') {
            let attributes = entry.split(' ').collect::<Vec<&str>>()[1];
            let _file_path= attributes.split('\0').collect::<Vec<&str>>()[0].to_string();
            let file_path = format!("{}/{}", repo, _file_path);
            let file_hash = attributes.split('\0').collect::<Vec<&str>>()[1].to_string();
            tree_hashmap.insert(file_path, file_hash);
        }
      }
      Ok(tree_hashmap)
}
pub fn get_tobe_commited_files(not_staged: &Vec<String>)->Result<Vec<String>, GitrError>{
    let working_dir_hashmap = get_working_dir_hashmap()?;
    let (index_hashmap, _) = get_index_hashmap()?;
    let current_commit_hashmap = get_current_commit_hashmap()?;
    let mut to_be_commited = Vec::new();
    for (path, hash) in index_hashmap.clone().into_iter() {
        if !current_commit_hashmap.contains_key(path.as_str()) {
            to_be_commited.push(path);
        }
        else if let Some(commit_hash) = current_commit_hashmap.get(path.as_str()) {
            if hash != *commit_hash  && !not_staged.contains(&path){ 
                to_be_commited.push(path);
            }
        }
    }
    for (path, _) in current_commit_hashmap.clone().into_iter() {
        if !working_dir_hashmap.contains_key(path.as_str()) {
            to_be_commited.push(path);
        }
    }
    Ok(to_be_commited)
}


/***************************
 *************************** 
 *    ADD FUNCTIONS
 **************************
 **************************/


pub fn save_and_add_blob_to_index(file_path: String) -> Result<(), GitrError> {
    let raw_data = file_manager::read_file(file_path.clone())?;
    let blob = Blob::new(raw_data)?;
    blob.save()?;
    let hash = blob.get_hash();
    file_manager::add_to_index(&file_path, &hash)?;
    Ok(())
}

pub fn update_index_before_add() -> Result<(),GitrError>{
    let repo = file_manager::get_current_repo()?;
    let index_path = &(repo.clone() + "/gitr/index");
    if Path::new(index_path).is_file() {
        let index_data = file_manager::read_index()?;
        let mut index_vector: Vec<&str> = Vec::new();
        if !index_data.is_empty() {
            index_vector = index_data.split('\n').collect::<Vec<&str>>();
        }
        let mut i: i32 = 0;
        while i != index_vector.len() as i32{
            let entry = index_vector[i as usize];
            let path_to_check = entry.split(' ').collect::<Vec<&str>>()[3];
            if !Path::new(path_to_check).exists(){
                index_vector.remove(i as usize);
                i -= 1;
            }
            i += 1;
        };
        file_manager::remove_file(format!("{}/gitr/index", repo))?;
        for entry in index_vector {
            let path = entry.split(' ').collect::<Vec<&str>>()[3];
            save_and_add_blob_to_index(path.to_string())?;
        }   
    }
    Ok(())
}


pub fn add_files_command(file_path:String)-> Result<(), GitrError>{
    let repo = get_current_repo()?;
    if file_path == "."{
        let files = visit_dirs(std::path::Path::new(&repo));
        for file in files{
            if file.contains("gitr"){
                continue
            }
            save_and_add_blob_to_index(file.clone())?;
        }
    }else{
        let full_file_path = repo + "/" + &file_path;
        save_and_add_blob_to_index(full_file_path)?;
    }
    Ok(())
}

/***************************
 *************************** 
 *   RM FUNCTIONS
 **************************
 **************************/

pub fn rm_from_index(file_to_delete: &str)->Result<bool, GitrError>{
    let mut removed:bool = false;
    let mut index = file_manager::read_index()?;
    index += "\n";
    let current_repo = file_manager::get_current_repo()?;
    let file_to_rm_path = format!("{}/{}", current_repo, file_to_delete);
    for line in index.lines(){
        let attributes = line.split(' ').collect::<Vec<&str>>();
        if attributes[3] == file_to_rm_path{
            let complete_line = format!("{}\n", line);
            index = index.replace(&complete_line, "");
            let res = index.trim_end().to_string();
            removed = true;
            let compressed_index = flate2compress(res)?;
            let _ = file_manager::write_compressed_data(&(current_repo +"/gitr/index"), compressed_index.as_slice());
            break
        }
    }
    Ok(removed)
}

/***************************
 *************************** 
 *    CLONE FUNCTIONS
 **************************
 **************************/

pub fn write_reference_from_cloning(references: Vec<(String, String)>, _ref_disc:String)-> Result<(), GitrError>{
    let repo = file_manager::get_current_repo()?;
    for reference in &references[1..]{
        let path_str = repo.clone() + "/gitr/"+ &reference.1.clone(); //ref path
        if references[0].0 == reference.0{
            file_manager::update_head(&reference.1.clone())?; //actualizo el head
        }
        let into_hash = reference.0.clone(); //hash a escribir en el archivo
        file_manager::write_file(path_str, into_hash)?; //escribo el hash en el archivo
    }
    Ok(())
}

pub fn clone_connect_to_server(address: String)->Result<TcpStream,GitrError>{
    let socket = match TcpStream::connect(address) {
        Ok(socket) => socket,
        Err(_) => return Err(GitrError::InvalidArgumentError("address".to_string(), "localhost:9418".to_string())),
    };
    Ok(socket)
}

pub fn clone_send_git_upload_pack(socket: &mut TcpStream)->Result<usize, GitrError>{
    match socket.write("0031git-upload-pack /mi-repo\0host=localhost:9418\0".as_bytes()){ //51 to hexa = 
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(GitrError::SocketError("clone_send_git_upload_pack()".to_string(), e.to_string())),
    }
}

pub fn clone_read_reference_discovery(socket: &mut TcpStream)->Result<String, GitrError>{
    let mut buffer = [0; 1024];
    let mut response = String::new();
    loop{
        let bytes_read = match socket.read(&mut buffer){
            Ok(bytes) => bytes,
            Err(e) => return Err(GitrError::SocketError("clone_read_reference_discovery()".to_string(), e.to_string())),
        };
        let received_message = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        if bytes_read == 0 || received_message == "0000"{ 
            break;
        }
        response.push_str(&received_message);
    }
    Ok(response)
}

pub fn write_socket(socket: &mut TcpStream, message: &[u8])->Result<(),GitrError>{
    match socket.write(message){
        Ok(_) => Ok(()),
        Err(e) => Err(GitrError::SocketError("write_socket()".to_string(), e.to_string())),
    }
}

pub fn read_socket(socket: &mut TcpStream, buffer: &mut [u8])->Result<(),GitrError>{
    let bytes_read = match socket.read(buffer){
        Ok(bytes) => bytes,
        Err(e) => return Err(GitrError::SocketError("read_socket()".to_string(), e.to_string())),
    };
    let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("String recibido de tamaño {}: {:?}", bytes_read, received_data);
    Ok(())
}


#[cfg(test)]
// Esta suite solo corre bajo el Git Daemon que tiene Bruno, está hardcodeado el puerto y la dirección, además del repo remoto.
mod tests{
    use crate::git_transport::ref_discovery::{self, assemble_want_message};

    use super::*;
    
    #[test]
    fn test00_clone_connects_to_daemon_correctly(){
        assert!(clone_connect_to_server("localhost:9418".to_string()).is_ok());
    }

    #[test]
    fn test01_clone_send_git_upload_pack_to_daemon_correctly(){
        let mut socket = clone_connect_to_server("localhost:9418".to_string()).unwrap();
        assert_eq!(clone_send_git_upload_pack(&mut socket).unwrap(),49); //0x31 = 49
    }
    
    #[test]
    fn test02_clone_receive_daemon_reference_discovery_correctly(){ //test viejo ya no corre
        let mut socket = clone_connect_to_server("localhost:9418".to_string()).unwrap();
        clone_send_git_upload_pack(&mut socket).unwrap();
        assert_eq!(clone_read_reference_discovery(&mut socket).unwrap(),"0103cf6335a864bda2ee027ea7083a72d10e32921b15 HEAD\0multi_ack thin-pack side-band side-band-64k ofs-delta shallow deepen-since deepen-not deepen-relative no-progress include-tag multi_ack_detailed symref=HEAD:refs/heads/main object-format=sha1 agent=git/2.34.1\n003dcf6335a864bda2ee027ea7083a72d10e32921b15 refs/heads/main\n");
    }

    #[test]	
    fn test03_clone_gets_reference_vector_correctly(){ //test viejo ya no corre
        let mut socket = clone_connect_to_server("localhost:9418".to_string()).unwrap();
        clone_send_git_upload_pack(&mut socket).unwrap();
        let ref_disc = clone_read_reference_discovery(&mut socket).unwrap();
        assert_eq!(ref_discovery::discover_references(ref_disc).unwrap(), 
        [("cf6335a864bda2ee027ea7083a72d10e32921b15".to_string(), "HEAD".to_string()), 
        ("cf6335a864bda2ee027ea7083a72d10e32921b15".to_string(), "refs/heads/main".to_string())]);
    }
    
    #[test]
    fn test04_clone_sends_wants_correctly(){
        let mut socket = clone_connect_to_server("localhost:9418".to_string()).unwrap();
        clone_send_git_upload_pack(&mut socket).unwrap();
        let ref_disc = clone_read_reference_discovery(&mut socket).unwrap();
        let references = ref_discovery::discover_references(ref_disc).unwrap();
        socket.write(assemble_want_message(&references,vec![]).unwrap().as_bytes()).unwrap();
    }
}