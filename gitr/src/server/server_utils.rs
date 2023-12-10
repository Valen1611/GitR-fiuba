extern crate flate2;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;

use std::thread;

use crate::commands::command_utils;
use crate::file_manager;
use crate::file_manager::contar_archivos_y_directorios;
use crate::git_transport::pack_file::create_packfile;
use crate::git_transport::pack_file::prepare_contents;
use crate::git_transport::pack_file::PackFile;

use crate::git_transport::ref_discovery;
use crate::gitr_errors::GitrError;
use crate::objects::commit::Commit;
use crate::objects::pull_request::PullRequest;

/// Pone en fucionamiento el Servidor Gitr en la direccion de socket provista. Maneja cada cliente de manera concurrente.
/// # Recibe
/// * s_addr: &str con la direccion del socket.
/// # Devuelve
/// Err(std::Error) si algun proceso interno tambien da error o no se pudo establecer bien la conexion.
pub fn server_init(s_addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(s_addr)?;
    let mut childs = Vec::new();

    thread::spawn(move || {
        let mut input = String::new();
        loop {
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let trimmed = input.trim().to_lowercase();
            if trimmed == "q" {
                // Envia un mensaje al hilo principal para indicar que debe salir
                let _ = TcpStream::connect("localhost:9418")
                    .unwrap()
                    .write("q".as_bytes())
                    .unwrap();
                break;
            }
            input.clear();
        }
    });
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut buf: [u8; 1] = [0; 1];
                let n = stream.peek(&mut buf)?;
                if n == 0 || buf[0] == b'q' {
                    break;
                }
                let builder = thread::Builder::new().name("cliente".to_string());
                childs.push(builder.spawn(|| handle_client(stream))?);
            }
            Err(e) => {
                eprintln!("Error al aceptar la conexión: {}", e);
            }
        }
    }
    for child in childs {
        match child.join() {
            Ok(result) => result?,
            Err(_e) => {
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    "Error en alguno de los hilos",
                ))
            }
        }
    }
    Ok(())
}

/// Maneja una conexion con cada cliente llevando a cabo el protocolo Git Transport o HTTP.
/// # Recibe
/// * stream: TcpStream ya conectado con el Gitr cliente
/// # Devuelve
/// Err(std::Error) si no se pudo establecer bien la conexion o algun proceso interno tambien da error.
fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];


    if let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            return Ok(());
        }
        // ########## HANDSHAKE ##########
        let request: String = String::from_utf8_lossy(&buffer[..n]).to_string();


        let ruta = request.split(' ').collect::<Vec<&str>>()[1].trim_start_matches('/');

        if request.starts_with("GET") {
            return handler_get_request(&ruta, stream);
        }
        if request.starts_with("POST") {
            println!("[SERVER]: POST request recieved:");
            println!("\x1b[34m{}\x1b[0m", request);
            /*
            Ejemplo para mandar con curl:
            curl -X POST -H "Content-Type: application/json" -d '{"id":1,"title":"titulo del pr","description":"descripcion del pr","head":"TestBranch2","base":"master","status":"open"}' localhost:9418/repos/sv/pulls
            */
            return handle_post_request(&ruta, &request, stream);
        }
        if request.starts_with("PUT") {
            println!("[SERVER]: PUT request recieved");
            /*
            Hacer el merge 💀💀💀
            PUT /repos/{repo}/pulls/{pull_number}/merge
            */
        }

        if request.starts_with("PATCH"){
            println!("[SERVER]: PATCH request recieved");
            return handle_patch_request(&request, stream);
        }
        
        
        //            PACKETLINE
        // ########## HANDSHAKE ##########
        
        return handle_pkt_line(request, stream)
       
        
    }
    Err(Error::new(
        std::io::ErrorKind::Other,
        "Error: no se pudo leer el stream",
    ))
}

fn handler_get_request(ruta: &str, mut stream: TcpStream) -> std::io::Result<()> {
    /*
    este caso puede ser
    Listar PRs - GET /repos/{repo}/pulls    ✅
    Obtener PR - GET /repos/{repo}/pulls/{pull_number}  ✅
    Listar commits - repos/{repo}/pulls/{pull_number}/commits ✅
    
    en Curl:
    curl -X GET localhost:9418/repos/sv/pulls
    curl -X GET localhost:9418/repos/sv/pulls/1
    */

    println!("[SERVER]: GET request recieved");
    
    let ruta_vec = ruta.split('/').collect::<Vec<&str>>();
    if ruta_vec.len() < 3 {
        println!("Error al parsear la ruta");
        stream.write("HTTP/1.1 422 ERROR\r\n\r\n".as_bytes())?;
        return Ok(());
    }
    let ruta_pulls = ruta_vec[..3].join("/");
    let ruta_repo_server = ruta_vec[..2].join("/");


    println!("route: {}", ruta); // ruta entera (sin el /repos)
    println!("route pulls: {}", ruta_pulls); // sv/pulls para obtener todos los PRs
    let prs: Vec<PullRequest> = match file_manager::get_pull_requests(ruta_pulls.clone()) {
        Ok(prs) => prs,
        Err(_) => {
            println!("Error al obtener PRs");
            stream.write("HTTP/1.1 422 ERROR\r\n\r\n".as_bytes())?;
            return Ok(());
        }
    };
    println!("PRs: {:?}", prs);
    
    let route_vec = ruta.split('/').collect::<Vec<&str>>();
    println!("rut vec: {:?}", route_vec);
    
    let last_dentry = route_vec[route_vec.len()-1];
    let mut response_body = String::new();
    for dentry in route_vec {
        println!("dentry: {}", dentry);

        if dentry == "pulls" && last_dentry == "pulls" {
            println!("estoy en el caso de listar todos los PRs");

            response_body.push_str("{");
            for pr in prs {
                let pr_str = match pr.to_string() {
                    Ok(pr_str) => pr_str,
                    Err(_) => {
                        println!("Error al parsear PR");
                        stream.write("HTTP/1.1 422 ERROR\r\n\r\n".as_bytes())?;
                        return Ok(());
                    }
                }; 

                response_body.push_str(&pr_str);
                response_body.push_str(",");
            }
            response_body.pop(); // saco la ultima coma
            response_body.push_str("}");
            break;
        }
        if dentry.parse::<u8>().is_ok() {
            println!("estoy en el caso de obtener un PR");
            let mut route_provisoria_corrected = ruta.clone();
            if last_dentry == "commits" {
                route_provisoria_corrected = ruta.trim_end_matches("/commits");
            }

            response_body = match file_manager::read_file(route_provisoria_corrected.clone().to_string()) {
                Ok(response_body) => response_body,
                Err(_) => {
                    println!("Error al obtener PR");
                    stream.write("HTTP/1.1 422 ERROR\r\n\r\nerror che".as_bytes())?;
                    return Ok(());
                }
            };
            if last_dentry == "commits" {
                println!("estoy en el caso de listar los commits de un PR");
                let pr = PullRequest::from_string(response_body.clone()).unwrap();
                let branch_name = pr.get_branch_name();
                println!("ruta_repo_server: {:?}", ruta_repo_server);
                let commits = match command_utils::branch_commits_list(branch_name.to_string(), ruta_repo_server.clone()) {
                    Ok(commits) => commits,
                    Err(_) => {
                        println!("Error al obtener commits");
                        stream.write("HTTP/1.1 422 ERROR\r\n\r\n".as_bytes())?;
                        return Ok(());
                    }
                };
                
                
                
                response_body = String::new();
                response_body.push_str("{");
                for commit in commits {
                    response_body.push_str(&commit);
                    response_body.push_str(",");
                }
                response_body.pop(); // saco la ultima coma
                response_body.push_str("}");
            }
        }
    
    }
    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", response_body);
    stream.write(response.as_bytes())?;
    return Ok(());
}

fn handle_post_request(ruta: &str, request: &str, mut stream: TcpStream) -> std::io::Result<()>{
    /*
    POST el body en la ruta
    
    Ejemplo de lo que se recibe, esto es el string request:

    POST /repos/reponame/pulls HTTP/1.1
    Host: localhost:9418
    User-Agent: curl/7.81.0
    Accept: 
    Content-Type: application/json
    Content-Length: 41

    {title:mensajito,head:branch,base:master} en realidad es el body de arriba,con este fallaria


    si quiseramos hacer que nuestro cliente mande un PR, deberia
    o ejecutar el comando ese de curl, o mandar el string asi con el formato
    como esta aca
     */
    

    // esta es la ruta provisoria porque todavia no tenemos
    // la carpeta /repos en el sv, cuano la tengamos
    // se usa directamente route y deberia andar todo igual

    // Y cuando tengamos esa carpeta, chequeemos que se esta
    // queriendo acceder a esa carpeta, si no, tiramos un
    // error 403 Forbidden

    
    // Nos fijamos cuantos PRs hay creados para asignar id al nuevo
    let id = match contar_archivos_y_directorios(&ruta){
        Ok(id) => id,
        Err(_) => {
            println!("Error al contar archivos y directorios");
            stream.write("HTTP/1.1 422 Validation failed\r\n\r\n".as_bytes())?;
            return Ok(());
        }
    };

    // Parseamos el body a un struct PullRequest 
    if request.split('\n').collect::<Vec<&str>>().len() < 8 {
        println!("Error al parsear el body");
        stream.write("HTTP/1.1 422 Validation failed\r\n\r\n".as_bytes())?;
        return Ok(());
    }

    let body = request.split('\n').collect::<Vec<&str>>()[7]; 
    let mut pull_request: PullRequest = match serde_json::from_str(&body) {
        Ok(pull_request) => pull_request,
        Err(_) => {
            println!("Error al parsear el body");
            println!("body: {}", body);
            stream.write("HTTP/1.1 422 Validation failed\r\n\r\n".as_bytes())?;
            return Ok(());
        }
    };
    
    match check_branches_exist(&pull_request, &ruta, &mut stream) {
        Ok(_) => {}
        Err(_) => {
            println!("Error al validar branches");
            stream.write("HTTP/1.1 422 Validation failed\r\n\r\n".as_bytes())?;
            return Ok(());
        }
    };

    // A la ruta le agregamos el id del PR
    let ruta = ruta.to_string() + "/" + id.to_string().as_str();
    
    // Le asignamos el id al PR
    pull_request.id = id as u8;

    // Creamos el PR            
    match file_manager::create_pull_request(&ruta, pull_request) {
        Ok(_) => println!("PR creado"),
        Err(_) => {
            println!("Error al crear PR");
            stream.write("HTTP/1.1 422 ERROR\r\n\r\n".as_bytes())?;
            return Ok(());
        }
          
    };

    stream.write("HTTP/1.1 201 OK\r\n\r\n".as_bytes())?;
    Ok(())
}

fn check_branches_exist(pull_request: &PullRequest, ruta: &str, stream: &mut TcpStream) -> Result<(), GitrError> {
    let branch_name = pull_request.get_branch_name();
    let base_name = pull_request.get_base_name();
    let ruta_repo_server = ruta.split('/').collect::<Vec<&str>>()[..2].join("/");
    let branches = match file_manager::get_branches(ruta_repo_server.clone()) {
        Ok(branches) => branches,
        Err(_) => {
            println!("Error al obtener branches");
            stream.write("HTTP/1.1 422 ERROR\r\n\r\n".as_bytes()).map_err(|_| GitrError::BranchNotFound)?;
            return Err(GitrError::BranchNotFound);
        }
    };
    if !branches.contains(&branch_name) {
        println!("No existe la branch head");
        stream.write("HTTP/1.1 422 Validation failed\r\n\r\n".as_bytes()).map_err(|_| GitrError::BranchNotFound)?;
        return Err(GitrError::BranchNotFound);
    }
    if !branches.contains(&base_name) {
        println!("No existe la branch base");
        stream.write("HTTP/1.1 422 Validation failed\r\n\r\n".as_bytes()).map_err(|_| GitrError::BranchNotFound)?;
        return Err(GitrError::BranchNotFound);
    }
    Ok(())
}

fn handle_patch_request(request: &str, mut stream: TcpStream) -> std::io::Result<()>{
    let route = request.split(' ').collect::<Vec<&str>>()[1];
    let route_provisoria = route.split('/').collect::<Vec<&str>>();
    let route_provisoria = route_provisoria[2..].join("/");
    if request.split('\n').collect::<Vec<&str>>().len() < 8 {
        println!("Error al parsear el body");
        stream.write("HTTP/1.1 422 Validation failed\r\n\r\n".as_bytes())?;
        return Ok(());
    }
    let id = route_provisoria.split('/').collect::<Vec<&str>>()[2];
    if !file_manager::pull_request_exist(&route_provisoria){
        println!("No existe el pull request solicitado");
        stream.write("HTTP/1.1 422 Validation failed\r\n\r\n".as_bytes())?;
        return Ok(());
    }
    let body = request.split('\n').collect::<Vec<&str>>()[7]; 
    let mut pull_request: PullRequest = match serde_json::from_str(&body) {
        Ok(pull_request) => pull_request,
        Err(_) => {
            println!("Error al parsear el body");
            println!("body: {}", body);
            stream.write("HTTP/1.1 422 Validation failed\r\n\r\n".as_bytes())?;
            return Ok(());
        }
    };
    match file_manager::create_pull_request(&route_provisoria, pull_request) {
        Ok(_) => println!("PR creado"),
        Err(_) => {
            println!("Error al crear PR");
            stream.write("HTTP/1.1 422 ERROR\r\n\r\n".as_bytes())?;
            return Ok(());
        }
          
    };
    stream.write("HTTP/1.1 201 OK\r\n\r\n".as_bytes())?;
    Ok(())

}

fn handle_pkt_line(request: String, mut stream: TcpStream) -> std::io::Result<()> {
    println!("estoy en pktline y recibi:\n {}", request);
    
    
    let guardados_id: HashSet<String>;
    let refs_string: String;
    match is_valid_pkt_line(&request) {
        Ok(_) => {}
        Err(_) => {
            let _ = stream.write("Error: no se respeta el formato pkt-line".as_bytes());
            return Ok(());
        }
    }
    let elems = split_n_validate_elems(&request)?;
    println!(
        "Comando: {}, Repo remoto: {}, host: {}",
        elems[0], elems[1], elems[2]
    );
    let r_path = "repos/".to_string() + elems[1];
    println!("Ruta del repositorio: {}", r_path);
    let _ = create_dirs(&r_path);
    // ########## REFERENCE DISCOVERY ##########
    (refs_string, guardados_id) = ref_discovery::ref_discovery(&r_path)?;
    let _ = stream.write(refs_string.as_bytes())?;
    // ########## ELECCION DE COMANDO ##########
    match elems[0] {
        "git-upload-pack" => {
            gitr_upload_pack(&mut stream, guardados_id, r_path)?;
        } // Mandar al cliente
        "git-receive-pack" => {
            gitr_receive_pack(&mut stream, r_path)?;
        } // Recibir del Cliente
        _ => {
            let _ = stream.write("Error: comando git no reconocido".as_bytes())?;
        }
    }
    return Ok(());
}

/// Lleva a cabo el protocolo Git Transport para el comando git-upload-pack, En el que se suben nuevos objetos al servidor.
/// Incluye packfile negotiation y el envio del packfile de ser necesario.
/// # Recibe
/// * stream: TcpStream ya conectado con el Gitr cliente
/// * guardados_id: HashSet con los ids de los objetos guardados en el servidor
/// * r_path: String con la ruta del repositorio del servidor
/// # Devuelve
/// Err(std::Error) si no se pudo establecer bien la conexion o algun proceso interno tambien da error.
fn gitr_upload_pack(
    stream: &mut TcpStream,
    guardados_id: HashSet<String>,
    r_path: String,
) -> std::io::Result<()> {
    // ##########  PACKFILE NEGOTIATION ##########
    let (wants_id, haves_id) = packfile_negotiation(stream, guardados_id)?;
    // ########## PACKFILE DATA ##########
    if !wants_id.is_empty() {
        snd_packfile(stream, wants_id, haves_id, r_path)?;
    }

    Ok(())
}

/// Lleva a cabo el protocolo Git Transport para el comando git-receive-pack, En el que se reciben nuevos objetos del cliente.
/// Incluye el Reference Update y el recibe el packfile de ser necesario.
/// # Recibe
/// * stream: TcpStream ya conectado con el Gitr cliente
/// * r_path: String con la ruta del repositorio del servidor
/// # Devuelve
/// Err(std::Error) si no se pudo establecer bien la conexion o algun proceso interno tambien da error.
fn gitr_receive_pack(stream: &mut TcpStream, r_path: String) -> std::io::Result<()> {
    // ##########  REFERENCE UPDATE ##########
    let mut buffer = [0; 1024];

    if let Ok(n) = stream.read(&mut buffer) {
        let (old, new, names) = get_changes(&buffer[..n])?;
        if old.is_empty() {
            //el cliente esta al dia
            return Ok(());
        }
        // ########## *PACKFILE DATA ##########
        if pkt_needed(old.clone(), new.clone()) {
            let (ids, content) = rcv_packfile_bruno(stream)?;
            update_contents(ids, content, r_path.clone())?;
        }
        update_refs(old, new, names, r_path)?;

        return Ok(());
    }
    Err(Error::new(
        std::io::ErrorKind::Other,
        "Error: no se pudo leer el stream",
    ))
}

/// Actualiza los contenidos de los objetos en el servidor, creando o modificando lo que sea necesario.
/// # Recibe
/// * ids: Vec<String> con los ids de los objetos a actualizar
/// * content: Vec<Vec<u8>> con los nuevos contenidos de los objetos a actualizar
/// * r_path: String con la ruta del repositorio del servidor
/// # Devuelve
/// Err(std::Error) si la longitud de los ids no se corresponde con la de los contenidos o si algun proceso interno tambien da error.
fn update_contents(ids: Vec<String>, content: Vec<Vec<u8>>, r_path: String) -> std::io::Result<()> {
    if ids.len() != content.len() {
        return Err(Error::new(
            std::io::ErrorKind::Other,
            "Error: no coinciden los ids con los contenidos",
        ));
    }
    for (i, id) in ids.into_iter().enumerate() {
        let dir_path = format!("{}/objects/{}", r_path.clone(), id.split_at(2).0);
        let _ = fs::create_dir(dir_path.clone());
        let mut archivo = File::create(&format!("{}/{}", dir_path, id.split_at(2).1))?;
        archivo.write_all(&content[i])?;
    }
    Ok(())
}

/// Envia el packfile al cliente.
/// # Recibe
/// * stream: TcpStream ya conectado con el Gitr cliente
/// * wants_id: Vec<String> con los ids de los commits o tags que el cliente quiere
/// * haves_id: Vec<String> con los ids de los objetos que el cliente tiene
/// * r_path: String con la ruta del repositorio del servidor
/// # Devuelve
/// Err(std::Error) si no se pudo preparar bien el packfile, si no se pudo obtener la data de alguno
/// de los objetos o si algun proceso interno tambien da error.
fn snd_packfile(
    stream: &mut TcpStream,
    wants_id: Vec<String>,
    haves_id: Vec<String>,
    r_path: String,
) -> std::io::Result<()> {
    let mut contents: Vec<Vec<u8>> = vec![];
    let all_commits =
        Commit::get_parents(wants_id.clone(), haves_id.clone(), r_path.clone()).unwrap_or(wants_id);
    let wants_id: Vec<String> =
        Commit::get_objects_from_commits(all_commits.clone(), haves_id, r_path.clone())
            .unwrap_or(vec![]);
    for id in wants_id.clone() {
        match file_manager::get_object_bytes(id, r_path.clone()) {
            Ok(obj) => contents.push(obj),
            Err(_) => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Error: no se pudo obtener el objeto",
                ))
            }
        }
    }
    if let Ok(pack) = pack_data_bruno(contents) {
        let _ = stream.write(&pack)?;
    } else {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "Algo salio mal\n",
        ));
    }
    Ok(())
}

/// Lleva a cabo el packfile negotiation con el cliente.
/// # Recibe
/// * stream: TcpStream ya conectado con el Gitr cliente
/// * guardados_id: HashSet con los ids de los objetos guardados en el servidor
/// # Devuelve
/// Una tupla con:
/// * wants_id: Vec<String> con los ids de los commits o tags que el cliente quiere
/// * haves_id: Vec<String> con los ids de los objetos que el cliente tiene
/// o un Error si algun proceso interno tambien da error o el cliente pide una referencia que el servidor no tiene.
fn packfile_negotiation(
    stream: &mut TcpStream,
    guardados_id: HashSet<String>,
) -> std::io::Result<(Vec<String>, Vec<String>)> {
    let (mut buffer, mut reply) = ([0; 1024], "0008NAK\n".to_string());
    let (mut wants_id, mut haves_id): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());

    let mut n = stream.read(&mut buffer)?;
    let mut buf = Vec::from(&buffer[..n]);
    while n == 1024 {
        buffer = [0; 1024];
        n = stream.read(&mut buffer)?;
        buf.append(&mut Vec::from(&buffer[..n]));
    }
    let pkt_line = from_utf8(&buf).unwrap_or("");
    if pkt_line == "0000" {
        return Ok((wants_id, haves_id));
    }
    (wants_id, haves_id) = wants_n_haves(pkt_line.to_string(), wants_id, haves_id)?;

    for want in wants_id.clone() {
        if !guardados_id.contains(&want) {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Error: not our ref: {}\n", want),
            ));
        }
    }
    for have in haves_id.clone() {
        if guardados_id.contains(&have) && reply == *"0008NAK\n" {
            reply = format!("003aACK {}\n", have.clone());
            let _ = stream.write(reply.as_bytes())?;
            break;
        }
    }
    if reply == *"0008NAK\n" {
        let _ = stream.write(reply.as_bytes())?;
    }
    Ok((wants_id, haves_id))
}

/// Recibe el packfile del cliente y lo descomprime.
/// # Recibe
/// * stream: TcpStream ya conectado con el Gitr cliente
/// # Devuelve
/// Una tupla con:
/// * hashes: Vec<String> con los ids de los objetos recibidos
/// * contents: Vec<Vec<u8>> con los contenidos de los objetos recibidos
/// O un Error si algun proceso interno tambien da error.
fn rcv_packfile_bruno(stream: &mut TcpStream) -> std::io::Result<(Vec<String>, Vec<Vec<u8>>)> {
    let mut buffer = Vec::new();
    let _ = stream.read_to_end(&mut buffer)?;
    let pack_file_struct = PackFile::new_from_server_packfile(&mut buffer);
    let pk_file = match pack_file_struct {
        Ok(pack_file) => pack_file,
        _ => {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "Error: no se pudo crear el packfile",
            ))
        }
    };
    let mut hashes: Vec<String> = Vec::new();
    let mut contents: Vec<Vec<u8>> = Vec::new();
    for object in pk_file.objects.iter() {
        hashes.push(object.get_hash());
        contents.push(object.get_data());
    }
    Ok((hashes, contents))
}

/// Verifica si es necesario enviar el packfile al cliente.
/// # Recibe
/// * old: Vec<String> con los ids de los objetos que el servidor tiene.
/// * new: Vec<String> con los ids de los objetos que el cliente quiere mandar.
/// # Devuelve
/// true si es necesario enviar el packfile, false en caso contrario.
fn pkt_needed(old: Vec<String>, new: Vec<String>) -> bool {
    let nul_obj = "0000000000000000000000000000000000000000";
    for i in 0..old.len() {
        if old[i] == nul_obj && new[i] != nul_obj {
            // crear referencia
            return true;
        } else if (new[i] == nul_obj && old[i] != nul_obj) || old[i] == new[i] {
            // borrar referencia o ref sin cambios
            continue;
        } else {
            // Modificacion de referencia
            return true;
        }
    }
    false
}

/// Actualiza las referencias del servidor.
/// # Recibe
/// * old: Vec<String> con los ids de los objetos que el servidor tiene.
/// * new: Vec<String> con los ids de los objetos que el cliente quiere mandar.
/// * names: Vec<String> con los nombres de las referencias que el cliente quiere mandar.
/// * r_path: String con la ruta del repositorio del servidor
/// # Devuelve
/// Err(std::Error) si no se pudo crear o borrar alguna referencia, si el nombre de alguna referencia
/// no es correcto o si algun proceso interno tambien da error.
fn update_refs(
    old: Vec<String>,
    new: Vec<String>,
    names: Vec<String>,
    r_path: String,
) -> std::io::Result<()> {
    let nul_obj = "0000000000000000000000000000000000000000";
    for i in 0..old.len() {
        let path = r_path.clone() + "/" + &names[i];
        if old[i] == nul_obj && new[i] != nul_obj {
            // crear referencia
            let mut new_file = File::create(&path)?;
            new_file.write_all(new[i].as_bytes())?;
            continue;
        } else if new[i] == nul_obj && old[i] != nul_obj {
            // borrar referencia
            fs::remove_file(&path)?;
            continue;
        } else if old[i] == new[i] {
            // no hubo cambios -> Error
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Error: el archivo no cambio",
            )); // no se si es el error correcto
        } else {
            // Modificacion de referencia
            let path = path.replace('\\', "/");
            let old_file = fs::File::open(&path)?;
            let mut old_ref = String::new();
            BufReader::new(old_file).read_line(&mut old_ref)?;
            if old_ref == old[i] {
                // si la ref vieja no cambio en el transcurso del programa -> ok
                let mut new_file = File::create(&path)?;
                new_file.write_all(new[i].as_bytes())?;
            } else {
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    "Error: nombre de archivo incorrecto",
                ));
            }
        }
    }
    Ok(())
}

/// Obtiene los cambios que el cliente quiere hacer en el servidor.
/// # Recibe
/// * buffer: &[u8] con los datos recibidos del cliente en el ref update request
/// # Devuelve
/// Una tupla con:
/// * old: Vec<String> con los ids de los objetos que el servidor tiene.
/// * new: Vec<String> con los ids de los objetos que el cliente quiere mandar.
/// * names: Vec<String> con los nombres de las referencias que el cliente quiere mandar.
/// O un Error si algun proceso interno tambien da error o si hay algun error en el formato
/// de los datos recibidos.
fn get_changes(buffer: &[u8]) -> std::io::Result<(Vec<String>, Vec<String>, Vec<String>)> {
    let changes = String::from_utf8_lossy(buffer); //.unwrap_or("Error");
    let mut old: Vec<String> = vec![];
    let mut new: Vec<String> = vec![];
    let mut names: Vec<String> = vec![];
    for change in changes.lines() {
        is_valid_pkt_line(&format!("{}\n", change))?;
        if change == "0000" {
            break;
        }
        let elems: Vec<&str> = change.split_at(4).1.split(' ').collect(); // [old, new, ref-name]
        if elems.len() != 3 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Error: Negociacion Fallida",
            ));
        }
        old.push(elems[0].to_string());
        new.push(elems[1].to_string());
        names.push(elems[2].to_string());
    }

    Ok((old, new, names))
}

/// Crea el packfile a partir de los contenidos de los objetos.
/// # Recibe
/// * contents: Vec<Vec<u8>> con los contenidos de los objetos a incluir en el packfile
/// # Devuelve
/// Vec<u8> con El packfile creado o un Error si algun proceso interno tambien da error.
fn pack_data_bruno(contents: Vec<Vec<u8>>) -> std::io::Result<Vec<u8>> {
    match create_packfile(prepare_contents(contents)) {
        Ok(pack) => Ok(pack),
        Err(_) => Err(Error::new(
            std::io::ErrorKind::Other,
            "Error: Armado de PACK fallido",
        )),
    }
}

/// Lleva a cabo el packfile negotiation con el cliente.
/// # Recibe
/// * requests: String con los datos recibidos del cliente en el packfile negotiation, (want y have lines)
/// * wants: Vec<String> con los ids de los commits o tags que el cliente quiere
/// * haves: Vec<String> con los ids de los objetos que el cliente tiene
/// # Devuelve
/// Una tupla con:
/// * wants: Vec<String> con los ids de los commits o tags que el cliente quiere
/// * haves: Vec<String> con los ids de los objetos que el cliente tiene
/// o un Error si algun proceso interno tambien da error.
fn wants_n_haves(
    requests: String,
    mut wants: Vec<String>,
    mut haves: Vec<String>,
) -> std::io::Result<(Vec<String>, Vec<String>)> {
    let mut nuls_cont = 0;
    for line in requests.lines() {
        is_valid_pkt_line(&(line.to_string() + "\n"))?;
        let elems: Vec<&str> = line.split_at(4).1.split(' ').collect(); // [want/have, obj-id]
        if nuls_cont == 0 {
            match elems[0] {
                "want" => wants.push(elems[1].to_string()),
                "" => {
                    nuls_cont += 1;
                } // 0000
                "0009done" => break,
                "0032have" => {
                    haves.push(elems[1].to_string());
                    nuls_cont += 1
                }
                _ => {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Error: Negociacion Fallida",
                    ))
                }
            }
        } else if nuls_cont == 1 {
            match elems[0] {
                "have" => haves.push(elems[1].to_string()),
                "" => nuls_cont += 1, // 0000
                "done" | "0009done" => break,
                _ => {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Error: Negociacion Fallida",
                    ))
                }
            }
        } else if nuls_cont == 2 {
            break;
        }
    }
    Ok((wants, haves))
}

/// Verifica si la linea de pkt-line recibida es valida.
/// # Recibe
/// * pkt_line: &str con la linea de pkt-line recibida
/// # Devuelve
/// Ok(()) si la linea es valida o un Error si no lo es.
fn is_valid_pkt_line(pkt_line: &str) -> std::io::Result<()> {
    if !pkt_line.is_empty()
        && pkt_line.len() >= 4
        && (usize::from_str_radix(pkt_line.split_at(4).0, 16) == Ok(pkt_line.len())
            || (pkt_line.starts_with("0000")
                && (pkt_line.split_at(4).1 == "\n"
                    || pkt_line.split_at(4).1.is_empty()
                    || is_valid_pkt_line(pkt_line.split_at(4).1).is_ok())))
    {
        return Ok(());
    }
    Err(Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "Error: No se sigue el estandar de PKT-LINE",
    ))
}

/// Separa los elementos de la linea de pkt-line en el Handshake.
/// # Recibe
/// * pkt_line: &str con la linea de pkt-line recibida
/// # Devuelve
/// Una lista con los elementos de la linea de pkt-line: (comando, repo_local, repo_remoto)
fn split_n_validate_elems(pkt_line: &str) -> std::io::Result<Vec<&str>> {
    let line = pkt_line.split_at(4).1;
    let div1: Vec<&str> = line.split(' ').collect();
    if div1.len() < 2 {
        return Err(Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "Error: No se sigue el estandar de PKT-LINE",
        ));
    }

    let div2: Vec<&str> = div1[1].split('\0').collect();
    let mut elems: Vec<&str> = vec![];
    if (div1.len() == 2) || div2.len() == 3 {
        elems.push(div1[0]);
        elems.push(div2[0].strip_prefix('/').unwrap_or(div2[0]));
        elems.push(div2[1].strip_prefix("host=").unwrap_or(div2[1]));
        return Ok(elems);
    }

    Err(Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "Error: No se sigue el estandar de PKT-LINE",
    ))
}

/// Crea los directorios y archivos necesarios para el repositorio del servidor.
/// # Recibe
/// * r_path: &str con la ruta del repositorio del servidor
/// # Devuelve
/// Err(std::io::Error) si algun proceso interno tambien da error o el repositorio ya existe.
fn create_dirs(r_path: &str) -> std::io::Result<()> {
    let p_str = r_path.to_string();
    fs::create_dir_all(p_str.clone())?;

    write_file(
        p_str.clone() + "/HEAD",
        "ref: refs/heads/master".to_string(),
    )?;
    fs::create_dir(p_str.clone() + "/refs")?;
    fs::create_dir(p_str.clone() + "/refs/heads")?;
    fs::create_dir(p_str.clone() + "/refs/tags")?;
    fs::create_dir(p_str.clone() + "/objects")?;
    fs::create_dir(p_str.clone() + "/pulls")?;
    Ok(())
}

/// Escribe un archivo con el texto provisto.
/// # Recibe
/// * path: String con la ruta del archivo a crear
/// * text: String con el texto a escribir en el archivo
/// # Devuelve
/// Err(std::io::Error) si algun proceso interno tambien da error.
fn write_file(path: String, text: String) -> std::io::Result<()> {
    let mut archivo = File::create(path)?;
    archivo.write_all(text.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::git_transport::pack_file;

    #[test]
    #[serial_test::serial]
    fn test02_split_n_validate() {
        let pkt_line = "0033git-upload-pack /project.git\0host=myserver.com\0".to_string();
        let elems = split_n_validate_elems(&pkt_line).unwrap();
        assert_eq!(elems[0], "git-upload-pack");
        assert_eq!(elems[1], "project.git");
        assert_eq!(elems[2], "myserver.com");
    }

    #[test]
    #[serial_test::serial]
    fn test03_is_valid_pkt_line() {
        assert!(is_valid_pkt_line("").is_err());
        assert!(is_valid_pkt_line("132").is_err());
        assert!(is_valid_pkt_line("0000hola").is_err());
        assert!(is_valid_pkt_line("kkkkhola").is_err());
        assert!(is_valid_pkt_line("0000").is_ok());
        assert!(is_valid_pkt_line("000ahola:)").is_ok());
        assert!(is_valid_pkt_line("0000").is_ok());
        assert!(is_valid_pkt_line("0032have 0123456789012345678901234567890123456789\n").is_ok());
        assert!(
            is_valid_pkt_line("00000032have 0123456789012345678901234567890123456789\n").is_ok()
        );
        assert!(is_valid_pkt_line("0033git-upload-pack /project.git\0host=myserver.com\0").is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test04_wants_n_haves() {
        let input = {
            "0032want 74730d410fcb6603ace96f1dc55ea6196122532d
0032want 7d1665144a3a975c05f1f43902ddaf084e784dbe
0032want 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a
0032want 7e47fe2bd8d01d481f44d7af0531bd93d3b21c01
0032want 74730d410fcb6603ace96f1dc55ea6196122532d
0000
0009done"
        };
        let (wants, haves) = wants_n_haves(input.to_string(), Vec::new(), Vec::new()).unwrap();
        assert_eq!(wants[0], "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(wants[1], "7d1665144a3a975c05f1f43902ddaf084e784dbe");
        assert_eq!(wants[2], "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a");
        assert_eq!(wants[3], "7e47fe2bd8d01d481f44d7af0531bd93d3b21c01");
        assert_eq!(wants[4], "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert!(haves.is_empty());
    }

    #[test]
    #[serial_test::serial]
    fn test05_get_changes() {
        let input = {
            "00677d1665144a3a975c05f1f43902ddaf084e784dbe 74730d410fcb6603ace96f1dc55ea6196122532d refs/heads/debug
006874730d410fcb6603ace96f1dc55ea6196122532d 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a refs/heads/master
0000"
        };
        let (old, new, names) = get_changes(input.as_bytes()).unwrap();
        assert_eq!(old[0], "7d1665144a3a975c05f1f43902ddaf084e784dbe");
        assert_eq!(old[1], "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(new[0], "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(new[1], "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a");
        assert_eq!(names[0], "refs/heads/debug");
        assert_eq!(names[1], "refs/heads/master");
    }

    #[test]
    #[serial_test::serial]
    fn test06_update_refs() {
        let r_path = "remote_repo";
        let _ = create_dirs(r_path);
        assert!(fs::metadata(format!("{}/refs/heads/debug", r_path)).is_err());
        assert!(fs::metadata(format!("{}/refs/heads/master", r_path)).is_err());
        // caso de creacion de archivo
        let old = vec![
            "0000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000".to_string(),
        ];
        let new = vec![
            "74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),
            "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_string(),
        ];
        let names = vec![
            "refs/heads/debug".to_string(),
            "refs/heads/master".to_string(),
        ];
        update_refs(old.clone(), new.clone(), names, r_path.to_string()).unwrap();
        assert!(pkt_needed(old, new));
        assert!(fs::metadata(format!("{}/refs/heads/debug", r_path)).is_ok());
        assert!(fs::metadata(format!("{}/refs/heads/master", r_path)).is_ok());
        assert_eq!(
            fs::read_to_string(format!("{}/refs/heads/debug", r_path)).unwrap_or("".to_string()),
            "74730d410fcb6603ace96f1dc55ea6196122532d"
        );
        assert_eq!(
            fs::read_to_string(format!("{}/refs/heads/master", r_path)).unwrap_or("".to_string()),
            "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a"
        );

        // caso modificacion de archivo
        let old = vec![
            "74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),
            "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_string(),
        ];
        let new = vec![
            "7d1665144a3a975c05f1f43902ddaf084e784dbe".to_string(),
            "74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),
        ];
        let names = vec![
            "refs/heads/debug".to_string(),
            "refs/heads/master".to_string(),
        ];
        update_refs(old.clone(), new.clone(), names, r_path.to_string()).unwrap();
        assert!(pkt_needed(old, new));
        assert!(fs::metadata(format!("{}/refs/heads/debug", r_path)).is_ok());
        assert!(fs::metadata(format!("{}/refs/heads/master", r_path)).is_ok());
        assert_eq!(
            fs::read_to_string(format!("{}/refs/heads/debug", r_path)).unwrap_or("".to_string()),
            "7d1665144a3a975c05f1f43902ddaf084e784dbe"
        );
        assert_eq!(
            fs::read_to_string(format!("{}/refs/heads/master", r_path)).unwrap_or("".to_string()),
            "74730d410fcb6603ace96f1dc55ea6196122532d"
        );
        // caso de borrado de archivo
        let old = vec![
            "7d1665144a3a975c05f1f43902ddaf084e784dbe".to_string(),
            "74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),
        ];
        let new = vec![
            "0000000000000000000000000000000000000000".to_string(),
            "0000000000000000000000000000000000000000".to_string(),
        ];
        let names = vec![
            "refs/heads/debug".to_string(),
            "refs/heads/master".to_string(),
        ];
        update_refs(old.clone(), new.clone(), names, r_path.to_string()).unwrap();
        assert!(!pkt_needed(old, new));
        assert!(fs::metadata(format!("{}/refs/heads/debug", r_path)).is_err());
        assert!(fs::metadata(format!("{}/refs/heads/master", r_path)).is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test07_update_contents_n_get_object() {
        let r_path = "remote_repo";
        let _ = create_dirs(r_path);
        let ids = vec![
            "74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),
            "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_string(),
        ];
        let content: Vec<Vec<u8>> = vec![
            pack_file::code("Hola mundo".to_string().as_bytes()).unwrap(),
            pack_file::code("Chau mundo".to_string().as_bytes()).unwrap(),
        ];
        update_contents(ids, content, r_path.to_string()).unwrap();
        assert_eq!(
            file_manager::get_object(
                "74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),
                r_path.to_string()
            )
            .unwrap(),
            "Hola mundo"
        );
        assert_eq!(
            file_manager::get_object(
                "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_string(),
                r_path.to_string()
            )
            .unwrap(),
            "Chau mundo"
        );
    }
}
