mod qq_reader;

use {
    crate::qq_reader::QqReader,
    std::{io, io::BufRead},
};

fn tests() {
    println!("{:?}", QqReader::new(",@".to_string()).read());
    println!("{:?}", QqReader::new(",foo ".to_string()).read());
    println!("{:?}", QqReader::new(",(1234 )".to_string()).read());
    println!(
        "{:?}",
        QqReader::new(",(think 123 thank thunk )".to_string()).read()
    );
}

fn main() {
    tests();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        println!("{:?}", QqReader::new(line.unwrap()).read());
    }
}
