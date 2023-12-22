mod qq_machine;

use {
    crate::qq_machine::QqMachine,
    std::{io, io::BufRead},
};

fn tests() {
    println!("{:?}", QqMachine::read(",@".to_string()));
    println!("{:?}", QqMachine::read(",foo ".to_string()));
    println!("{:?}", QqMachine::read(",(1234 )".to_string()));
    println!(
        "{:?}",
        QqMachine::read(",(think 123 thank thunk )".to_string())
    );
}

fn main() {
    tests();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        println!("{:?}", QqMachine::read(line.unwrap()));
    }
}
