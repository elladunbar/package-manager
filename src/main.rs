use package_manager::{Manager, pacman::*};

fn main() {
    let m = Pacman;
    print!("{:?}", m.manager_path().unwrap());
}
