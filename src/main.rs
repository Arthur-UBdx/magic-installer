use magic_installer::*;

fn main() {
    let config = Config::load();
    config.print();
    let path = config.get_env_path();
    println!("{}", path);
}

//TODO 
// - Add a way to change config.
// - Error Message when not connected to internet.
