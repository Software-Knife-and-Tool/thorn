mod bq_machine;

use std::{io, io::BufRead};

use crate::bq_machine::BqMachine;

fn tests() {
    BqMachine::parse("`,@".to_string());
    BqMachine::parse("`,foo ".to_string());
    BqMachine::parse("`,(1234 )".to_string());
    BqMachine::parse("`,(think 123 thank thunk )".to_string());
}

fn main() {
    tests();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        BqMachine::parse(line.unwrap());
    }
}
