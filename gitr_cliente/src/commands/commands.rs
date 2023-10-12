/*
    NOTA: Puede que no todos los comandos requieran de flags,
    si ya esta hecha la funcion y no se uso, se puede borrar
    (y hay que modificar el llamado desde handler.rs tambien)
*/


pub fn hash_object(flags: Vec<String>) {
    println!("hash_object");
}

pub fn cat_file(flags: Vec<String>) {
    println!("cat_file");
}

pub fn init(flags: Vec<String>) {
    println!("init");
}

pub fn status(flags: Vec<String>) {
    println!("status");
}

pub fn add(flags: Vec<String>) {
    println!("add");
    println!("flags: {:?}", flags);
}

pub fn rm(flags: Vec<String>) {
    println!("rm");
} 

pub fn commit(flags: Vec<String>) {
    println!("commit");
}

pub fn checkout(flags: Vec<String>) {
    println!("checkout");
}

pub fn log(flags: Vec<String>) {
    println!("log");
}

pub fn clone(flags: Vec<String>) {
    println!("clone");
}

pub fn fetch(flags: Vec<String>) {
    println!("fetch");
}

pub fn merge(flags: Vec<String>) {
    println!("merge");
}

pub fn remote(flags: Vec<String>) {
    println!("remote");
}

pub fn pull(flags: Vec<String>) {
    println!("pull");
}

pub fn push(flags: Vec<String>) {
    println!("push");
}

pub fn branch(flags: Vec<String>) {
    println!("branch");
}
