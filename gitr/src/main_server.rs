use gitr::server::server_utils;

fn main(){
    let address =  "localhost:9418";
    let ruta = "remote_repo";
    println!("Server inicializado en {}", address);
    let _ = server_utils::server_init(ruta, address);
}