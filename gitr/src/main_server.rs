use gitr::server::server_utils;

fn main(){
    let address =  "localhost:9418";
    println!("Server inicializado en {}", address);
    let _ = server_utils::server_init(address);
}