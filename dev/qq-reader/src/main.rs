mod reader;

use {
    crate::reader::QqReader,
    std::{io, io::BufRead},
};

// verified by ssbcl
fn tests() {
    println!("{:?}", QqReader::new("`,@".to_string()).read());
    println!("{:?}", QqReader::new("`,pi".to_string()).read());
    println!("{:?}", QqReader::new("`,1234".to_string()).read());
    println!("{:?}", QqReader::new("`,(+ 1 2)".to_string()).read());
    println!(
        "{:?}",
        QqReader::new("`,(list 'think 123 'thank 'thunk)".to_string()).read()
    );
}

fn main() {
    tests();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        println!("{:?}", QqReader::new(line.unwrap()).read());
    }
}
