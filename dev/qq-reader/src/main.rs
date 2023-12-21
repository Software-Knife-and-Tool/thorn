mod qq_machine;

use std::{io, io::BufRead};

use crate::qq_machine::QqMachine;

fn tests() {
    QqMachine::parse("`,@".to_string());
    QqMachine::parse("`,foo ".to_string());
    QqMachine::parse("`,(1234 )".to_string());
    QqMachine::parse("`,(think 123 thank thunk )".to_string());
}

fn main() {
    tests();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        QqMachine::parse(line.unwrap());
    }
}
