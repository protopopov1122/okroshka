use std::io;
use serde_json;

pub mod okroshka;

fn main() {
    let module: okroshka::ir::module::IRModule =
        serde_json::from_reader(io::stdin()).unwrap();

    println!("{:?}", module);
}
