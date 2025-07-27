use package_manager::{Manager, pacman::Pacman};
use std::env;

fn main() {
    let arg = env::args().nth(1).unwrap();
    let mut packages = Pacman::remote_search(&arg).unwrap();
    packages.sort_by(|p1, p2| p1.name.cmp(&p2.name));
    for package in packages {
        println!("{}", package);
    }
}
