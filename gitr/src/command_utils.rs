use std::{io::{Write, Read}, fs::{self}, path::Path, collections::HashMap, net::TcpStream};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Sha1, Digest};
use crate::{file_manager::{read_index, self, get_head, get_current_commit, get_current_repo, visit_dirs, update_working_directory}, objects::git_object::GitObject, diff::{Diff, self}};
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
    return Ok((object_hash.to_string(), res_output.clone(), size.to_string(), object_type.to_string()));
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
    return Ok(())
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
    return Ok(())
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
    let branch_name = head.split("/").collect::<Vec<&str>>()[2];
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
    // let mut files_raw_data = Vec::new();

    // for blob in blobs.clone() {
    //     let blob_data = file_manager::read_object(&blob)?;
    //     files_raw_data.push(blob_data);
    // }
    
    Ok(())
}

/// Toma el path de un archivo y los diffs que hay que aplicarle. (No detecta conflicts)
/// Crea el archivo con los cambios aplicados.
fn aplicar_difs(path: String, diff: Diff)-> Result<(), GitrError> {
    let string_archivo = file_manager::read_file(path.clone())?;
    let mut archivo_reconstruido = vec![];
    println!("string_archivo: {:?}", string_archivo);
    for (i,line) in string_archivo.lines().enumerate(){
        println!("");
        println!("");
        println!("Base[{}]: {}", i, line);
        let tiene_add = diff.has_add_diff(i);
        if diff.has_delete_diff(i){ //sí y la linea se tiene que agregar
            print!(". Hay dif de delete. ");
            if tiene_add.0{
                print!(". Hay dif de add. Pusheo: {}",tiene_add.1.clone());
                archivo_reconstruido.push(tiene_add.1.clone()+"\n"); //luego el agregado
            }
            print!(". No hay diff de add.");
            continue;
        }
        else if tiene_add.0 { //sí y se elimina, no la pusheo porque la tengo que borrar, la salteo
            print!(". No hay dif de delete. Sí hay de add. Agrego: [{},{}]",line.to_string().clone(),tiene_add.1.clone());
            archivo_reconstruido.push(line.to_string()+"\n"); //primero el base
            archivo_reconstruido.push(tiene_add.1.clone()+"\n"); //luego el agregado
        } else {
            archivo_reconstruido.push(line.to_string()+"\n"); //primero el base
        }
    }
    println!("archivo_reconstruido: {:?}", archivo_reconstruido);
    file_manager::write_file(path+"_mergeado", archivo_reconstruido.concat().to_string())?;
    Ok(())
}

fn comparar_diffs(diff_base_origin: Diff, diff_base_branch: Diff) -> Result<(), GitrError> {
    let (mut i, mut j) = (0,0);
    let mut diff_final = Diff::new("".to_string(), "".to_string());
    
    let origin = diff_base_origin.lineas.clone();
    let new = diff_base_branch.lineas.clone();
    
    loop {
        if origin[i] == new[j]{
            //conflict
        }
        else{
            if origin[i].0 < new[j].0{
                diff_final.lineas.push(origin[i].clone());
                // lo hacemos con lineas
                // o pateamos el refactor y lo armamanos
                // a mano con las lines agregadsa y eliminadas??


                i+=1;
            }
            else{
                diff_final.lineas.push(new[j].clone());
                j+=1;
            } 
        }
       
    }

                                    

    Ok(())
}


pub fn three_way_merge(base_commit: String, origin_commit: String, branch_commit: String) -> Result<(), GitrError> {
    println!("entro a three way");

    let branch_hashmap = get_commit_hashmap(branch_commit.clone())?;
    let mut origin_hashmap: HashMap<String, String> = get_commit_hashmap(origin_commit.clone())?;
    file_manager::add_new_files_from_merge(origin_hashmap.clone(), branch_hashmap.clone())?;
    origin_hashmap = get_commit_hashmap(origin_commit.clone())?;
    let base_hashmap = get_commit_hashmap(base_commit.clone())?;

    //IDEA: Agarrar todos los archivos y carpetas de branch y crearlos en el working dir.
    /*
    base_file_data: "hola\ncambios en otra"
    origin_file_data: "hola y chau en master\n"
    branch_file_data: "hola\n"
    
     */
    

    for (path, origin_file_hash) in origin_hashmap.iter(){

        let origin_file_data =file_manager::read_file(path.clone())?; 
        
        
        if branch_hashmap.contains_key(&path.clone()){
            let branch_file_hash = branch_hashmap[path].clone(); //aax
            let branch_file_data = file_manager::read_file_data_from_blob_hash(branch_file_hash.clone())?;

            
            if origin_file_hash == &branch_file_hash{
                // base     origin     branch    result
                //          aaa        aaa       aaa
                continue;
            }
            
            let base_file_hash = base_hashmap[path].clone(); // chequear que capaz puede no exisiir en base
            let base_file_data = file_manager::read_file_data_from_blob_hash(base_file_hash.clone())?;

            if &base_file_hash == origin_file_hash {
                // base     origin     branch    result
                // aaa      aaa        aax       aaxa



                //me quedo con branch
                //saco el diff entre branch y base

                let diff_base_branch = Diff::new(base_file_data, branch_file_data);
                aplicar_difs(path.clone(), diff_base_branch)?;
                continue;
            }
            
            if base_file_hash == branch_file_hash {
                // base     origin     branch    result
                // aaa      aax        aaa       aax

                //me quedo con origin porque en branch no hubo cambios nuevos ahi                
                continue;
           
            }

            // aca se modifico el mismo archivo en ambas ramas (origin y branch)
            // con respecto a base
            
            //      base     origin     branch    result
            //      aaa      aax        aaz       

            // si los indices de los diffs, coinciden, hay conflicts
            // si no coinciden, solamente se mergean los cambios
            
            println!("aca si deberia caer");
            println!("base_file_data: {:?}", base_file_data);
            println!("origin_file_data: {:?}", origin_file_data);
            println!("branch_file_data: {:?}", branch_file_data);

            let diff_base_origin = Diff::new(base_file_data.clone(), origin_file_data.clone());
            let diff_base_branch = Diff::new(base_file_data, branch_file_data);


            // no necesriamente hay conflicts
            // aca necesitamos una funcion que compare los diffs
            // y si devuelve un Diff, lo aplicamos y listo
            // si devuelve un DiffConflict... tambien lo aplicamos y listo
            // pero con su logica de conflictos

            let union_diffs = comparar_diffs(diff_base_branch, diff_base_origin)?;

            // match union_diffs {
            //     /*
            //     hacer eso de match Diff
            //     o match DiffConclit como se hace con los GitObjects
                
            //      */
            // }

            


            /*
            (1, hola, chau)
            (2,---,agregar algo)
            (3, chau, )
            4 ---
            

            
             */
            /*
             <<<<<<<<,
            generar_incoming() = Vec[string],
             ==========,
            generar_actual() = Vec[String],
             >>>>>,
             .concat()
             */

            
            /*

            
            conflit



            */

        }
        else{
            continue;
        }
    }

    //algoritmo merge git
    //1)diff origin vs base
    //2)diff branch vs base
        //2bis) si branch tiene archivos nuevos le hago add.
    //3) aplicar los diffs a los archivos en base
    //4) merge commit

/*
hashmap: 
{"nuevito/otra": "5e8cf4544ba6d6bafa636e0b38bdc2277da8bef1", 
"nuevito/hola": "13fd0cb913ab1725abab00937407c4e046314228", 
"nuevito/otra/ola": "6fcf9a84b35d93e984047e3c7e8418289ec09f19"}
*/
    /*
    base    origin  branch      result
    
    aaa     aaa     aax         aax                     diff={}U{-a(3) +x(3)}
    bbb     bbx     bbb         bbx                     diff={-b(3) +x(3)}U{}
    ccc     ccx     ccz         ver diferencias         diff={-c(3) +x(3)}U{-c(3) +z(3)} ---> conflict y tenes que elegir
            DDD     
    

    O--N--O--M
   \      /
    X--M-/
     */



    
    Ok(())
}


/***************************
 *************************** 
 *   STATUS FUNCTIONS
 **************************
 **************************/

 pub fn get_working_dir_hashmap() -> Result<HashMap<String, String>, GitrError>{
    // working dir
    let mut working_dir_hashmap = HashMap::new();
    //busco el working dir
    let repo = file_manager::get_current_repo()?;
    let path = Path::new(repo.as_str());
    let files= visit_dirs(path);
    //cargo el diccionario
    for file_path in files {
        let file_data = file_manager::read_file(file_path.clone())?;
        
        let blob = Blob::new(file_data.clone())?;
        let hash = blob.get_hash();
        working_dir_hashmap.insert(file_path, hash);
    }
    Ok(working_dir_hashmap)
}

pub fn status_print_to_be_comited(to_be_commited: &Vec<String>){
    if !to_be_commited.is_empty() {
        println!("Changes to be committed:");
        println!("  (use \"rm <file>...\" to unstage)");

        for file in to_be_commited.clone() {
            let file_name = match file.split_once ('/'){
                Some((_path, file)) => file.to_string(),
                None => file.to_string(),
            };
            println!("\t\x1b[92mmodified   {}\x1b[0m", file_name);
        }
    }
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
    Ok(())
}


pub fn get_index_hashmap() -> Result<(HashMap<String, String>, bool), GitrError> {
    // index
    let mut index_hashmap = HashMap::new();
    //busco el index
    let mut hayindex = true;
    let index_data = match file_manager::read_index() {
        Ok(data) => data,
        Err(_) => {
            //let message = format!("\nNo commits yet\n\nnothing to commit (create/copy files and use \"git add\" to track)");
            //println!("{}", message);
            hayindex = false;
            String::new()
        }
    };
    // cargo el diccionario
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

pub fn get_subtrees_data(hash_of_tree_to_read: String, file_path: String, mut tree_hashmap: &mut HashMap<String, String>) -> Result<(), GitrError>{
    let tree_data = file_manager::read_object(&hash_of_tree_to_read)?;

    let tree_entries = match tree_data.split_once('\0') {
        Some((_tree_type, tree_entries)) => tree_entries,
        None => "",
    };
    // cargo el diccionario
    for entry in tree_entries.split('\n') {
        if entry.split(' ').collect::<Vec<&str>>()[0] == "40000"{
            let attributes = entry.split(' ').collect::<Vec<&str>>()[1];
            let relative_file_path= attributes.split('\0').collect::<Vec<&str>>()[0].to_string();
            let file_path = format!("{}/{}", file_path, relative_file_path);
            let file_hash = attributes.split('\0').collect::<Vec<&str>>()[1].to_string();
            get_subtrees_data(file_hash, file_path, &mut tree_hashmap)?;
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


/// only blobs
pub fn get_commit_hashmap(commit: String) -> Result<HashMap<String, String>, GitrError> {
      // current commit
      let mut tree_hashmap = HashMap::new();
    let current_commit = get_current_commit()?;
    if current_commit == commit{
        let (index_hashmap, _) = get_index_hashmap()?;
        return Ok(index_hashmap);
    }
      //busco el commit
      if !commit.is_empty() {
        
        let repo = file_manager::get_current_repo()?;
        let tree = file_manager::get_main_tree(commit)?;
        let tree_data = file_manager::read_object(&tree)?;
        let tree_entries = match tree_data.split_once('\0') {
            Some((_tree_type, tree_entries)) => tree_entries,
            None => "",
        };
          // cargo el diccionario
          
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

pub fn clone_connect_to_server(address: String)->Result<TcpStream,GitrError>{
    let socket = match TcpStream::connect(address) {
        Ok(socket) => socket,
        Err(_) => return Err(GitrError::InvalidArgumentError("address".to_string(), "localhost:9418".to_string())),
    };
    Ok(socket)
}

pub fn clone_send_git_upload_pack(socket: &mut TcpStream)->Result<usize, GitrError>{
    // let msj = format!("git-upload-pack /{}\0host={}\0",file_manager::get_current_repo()?, file_manager::get_remote()?);
    // let msj = format!("{:04x}{}", msj.len() + 4, msj);    
    // match socket.write(msj.as_bytes()){ //51 to hexa = 
    //     Ok(bytes) => Ok(bytes),
    //     Err(e) => Err(GitrError::ConnectionError),
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
mod tests_clone{
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
    fn test02_clone_receive_daemon_reference_discovery_correctly(){
        let mut socket = clone_connect_to_server("localhost:9418".to_string()).unwrap();
        clone_send_git_upload_pack(&mut socket).unwrap();
        assert_eq!(clone_read_reference_discovery(&mut socket).unwrap(),"0103cf6335a864bda2ee027ea7083a72d10e32921b15 HEAD\0multi_ack thin-pack side-band side-band-64k ofs-delta shallow deepen-since deepen-not deepen-relative no-progress include-tag multi_ack_detailed symref=HEAD:refs/heads/main object-format=sha1 agent=git/2.34.1\n003dcf6335a864bda2ee027ea7083a72d10e32921b15 refs/heads/main\n");
    }

    #[test]	
    fn test03_clone_gets_reference_vector_correctly(){
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

#[cfg(test)]
mod tests_merge{
    use super::*;
    use crate::commands::*;
    
    #[test]
    fn test00_three_way_merge(){
        let commit_master = "ad43f12629f80764e4d5217537a12de193c48f15".to_string();
        let commit_branch = "3b7ddd0af8f07fd7c3b08bda494c1d9f1929fa3d".to_string();
        commands::merge(vec!["c1231533842cda7bf87bda8410cc688e8876134a".to_string()]).unwrap();
    }
}
