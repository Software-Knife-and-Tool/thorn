#![allow(dead_code)]

use {rust_fsm::*, std::result::Result};

pub struct QqMachine {}

#[derive(Debug)]
pub enum QqExpr {
    Form(String),       // plain form
    Quote(String),      // quoted form
    Dot(String),        // dotted list
    List(Box<QqExpr>),  // make list of
    QQuote(Vec<QqExpr>),
}

state_machine! {
    derive(Debug)
    repr_c(true)
    Reader(QuasiQuote)

    // `
    QuasiQuote => {
        Comma => Comma,               // `,
        Constant => Exit [ Form ],    // `basic
        List => List,                 // `(
        QuasiQuote => QuasiQuote,     // ``
        Symbol => Exit [ Quote ],     // `basic
    },

    // `,
    Comma => {
        Constant => Exit [ Form ],    // `,basic
        List => CommaList,            // `,(
        QuasiQuote => QuasiQuote,     // `,`
        Symbol => Exit [ Form ],      // `,basic
    },

    // `(
    List => {
        Comma => CommaList,           // `(,
        Constant => List [ Form ],    // `(basic
        EndList => Exit [ EndList ],  // `()
        List => List,                 // `((
        QuasiQuote => QuasiQuote,     // `(`
        Symbol => List [ Quote ],     // `(basic
    },

    // `,(
    CommaList => {
        Constant => CommaList [ Form ],            // `,(basic
        EndList => Exit [ EndList ],               // `,()
        List => List,                              // `,((
        QuasiQuote => CommaList [ QuasiQuote ],    // `,(`
        Symbol => CommaList [ Quote ],             // `,(basic
    },

    // `,(,
    CommaInList => {
        At => CommaInList [ At ] ,
        Comma => CommaList,
        Constant => CommaList [ Form ],
        EndList => Exit [ EndList ],
        List => List,
        QuasiQuote => CommaList [ QuasiQuote ],
        Symbol => CommaList [ Quote ],
    },
}

impl QqMachine {
    pub fn read(source: String) -> Result<String, String> {
        match Self::parse(source) {
            Ok(vec) => Ok(Self::compile(vec)),
            Err(e) => Err(e),
        }
    }
    
    pub fn compile(list: Vec<QqExpr>) -> String {
        let mut out = "".to_string();
        
        println!("compile: {:?}", list);

        for el in list {
            match el {
                QqExpr::Form(expr) => out.push_str(&format!(" {}", &expr)),
                QqExpr::Quote(expr) => out.push_str(&format!(" (:quote {})", &expr)),
                QqExpr::Dot(expr) => out.push_str(&format!(" . {}", &expr)),
                QqExpr::List(expr) => out.push_str(&format!(" (mu:cons {:?} ())", &*expr)),
                QqExpr::QQuote(exprvec) => out.push_str(&Self::compile(exprvec)),
            }
        }

        out
    }
    
    pub fn parse(mut source: String) -> Result<Vec<QqExpr>, String> {
        println!("parse: `{}", source);
        
        let mut read_char = || -> Option<char> {
            if source.is_empty() {
                None
            } else {
                Some(source.remove(0))
            }
        };

        let mut next_state = || -> Option<(ReaderInput, String)> {
            match read_char() {
                None => return None,
                Some(ch) => match ch {
                    '(' => Some((ReaderInput::List, "(".to_string())),
                    ')' => Some((ReaderInput::EndList, ")".to_string())),
                    '`' => Some((ReaderInput::QuasiQuote, "`".to_string())),
                    ',' => Some((ReaderInput::Comma, ",".to_string())),
                    '@' => Some((ReaderInput::At, "@".to_string())),
                    _ => {
                        let mut token = String::from(ch);

                        loop {
                            match read_char() {
                                None => break,
                                Some(ch) => {
                                    if ch.is_digit(10) || ch.is_alphabetic() {
                                        token.push(ch)
                                    } else {
                                        // unread_char(ch);
                                        break;
                                    }
                                }
                            }
                        }

                        if ch.is_alphabetic() {
                            Some((ReaderInput::Symbol, token))
                        } else {
                            Some((ReaderInput::Constant, token))
                        }
                    }
                },
            }
        };

        let mut expansion: Vec<QqExpr> = vec![];
        let mut machine: StateMachine<Reader> = StateMachine::new();

        loop {
            match next_state() {
                None => return Err("unterminated expression.".to_string()),
                Some((state, token)) => {
                    match machine.consume(&state) {
                        Err(_) => {
                            return Err(format!(
                                "syntax, token {:?} in state {:?}",
                                token,
                                machine.state(),
                            ));
                        }
                        Ok(output) => {
                            let new_state = machine.state();

                            // println!("  [ {:?} {} ] enters {:?}", state, token, new_state);
                            match new_state {
                                ReaderState::QuasiQuote => {
                                    // Self::parse(source);
                                }
                                ReaderState::Comma => {
                                }
                                ReaderState::CommaList => match output {
                                    None => (),
                                    Some(qualifier) => {
                                        /*
                                        println!(
                                            "  [ {:?} {:?} {} ] enters {:?}",
                                            state, qualifier, token, new_state
                                    );
                                        */
                                        match qualifier {
                                            ReaderOutput::Form => {
                                                expansion.push(QqExpr::Form(token))
                                            }
                                            ReaderOutput::Quote => {
                                                expansion.push(QqExpr::Quote(token))
                                            }
                                            ReaderOutput::EndList => break,
                                            _ => {
                                                return Err(
                                                    "unimplemented CommaList element".to_string()
                                                )
                                            }
                                        }
                                    }
                                },
                                ReaderState::List => match output {
                                    None => (),
                                    Some(qualifier) => {
                                        /*
                                        println!(
                                            "  ( [ {:?} {:?} {} ] enters {:?}",
                                            state, qualifier, token, new_state
                                    );
                                        */
                                        match qualifier {
                                            ReaderOutput::Form => {
                                                expansion.push(QqExpr::Form(token))
                                            }
                                            ReaderOutput::Quote => {
                                                expansion.push(QqExpr::Quote(token))
                                            }
                                            ReaderOutput::EndList => break,
                                            _ => {
                                                return Err(
                                                    "unimplemented List element".to_string()
                                                )
                                            }
                                        }
                                    }
                                },
                                ReaderState::Exit => {
                                    let qualifier = output.unwrap();
                                    /*
                                    println!(
                                        "  [ {:?} {:?} {} ] enters {:?}",
                                        state, qualifier, token, new_state
                                );
                                    */

                                    match qualifier {
                                        ReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                        ReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                        ReaderOutput::EndList => break,
                                        _ => return Err("unimplemented Exit element".to_string()),
                                    }
                                    break;
                                }
                                _ => {
                                    println!("  unimplemented state [ {:?} {} ]", state, token);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(expansion)
    }
}
