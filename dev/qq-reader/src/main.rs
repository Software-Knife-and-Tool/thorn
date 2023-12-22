mod qq_machine;

use std::{io, io::BufRead};

use crate::qq_machine::QqMachine;

fn tests() {
    println!("{:?}", QqMachine::parse("`,@".to_string()));
    println!("{:?}", QqMachine::parse("`,foo ".to_string()));
    println!("{:?}", QqMachine::parse("`,(1234 )".to_string()));
    println!("{:?}", QqMachine::parse("`,(think 123 thank thunk )".to_string()));
}

fn main() {
    tests();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        println!("{:?}", QqMachine::parse(line.unwrap()));
    }
}
