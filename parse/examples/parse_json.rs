//! Example: parse JSON from stdin or a file.

use std::env;
use std::fs;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = if let Some(path) = env::args().nth(1) {
        fs::read_to_string(path)?
    } else {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    };
    let v = parse::json::parse_str(&data)?;
    println!("{:#?}", v);
    Ok(())
}
