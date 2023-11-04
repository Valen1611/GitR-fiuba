fn hello_world() {
    println!("Hello, world!");
}
fn main() {
    hello_world();
    let farm = Farm::new("My Farm");
    let farmer = Farmer::new("John", farm);
    farmer.live_on_farm();
}

struct Farm {
    name: String,
}

impl Farm {
    fn new(name: &str) -> Farm {
        Farm { name: name.to_string() }
    }
}

struct Farmer<'a> {
    name: &'a str,
    farm: Farm,
}

impl<'a> Farmer<'a> {
    fn new(name: &'a str, farm: Farm) -> Farmer<'a> {
        Farmer { name, farm }
    }

    fn live_on_farm(&self) {
        println!("{} is living on {} farm!", self.name, self.farm.name);
    }
}
