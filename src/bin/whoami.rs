use std::env;

pub fn main() {
    println!("{}", env::var("USER").unwrap_or(String::new()));
}
