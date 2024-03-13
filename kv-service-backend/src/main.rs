pub mod key_value_service {
    tonic::include_proto!("keyvalueservice");
}

mod services;
mod utils;

fn main() {
    println!("Hello, world!");
}
