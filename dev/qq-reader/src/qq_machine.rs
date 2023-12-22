#![allow(dead_code)]

use rust_fsm::*;

pub struct QqMachine {}

#[derive(Debug)]
pub enum QqExpr {
    Form(String),      // plain form
    Quote(String),     // quoted form
    Dot(String),       // dotted list
    // List(QqExpr),      // inner form
    QQuote(Vec<QqExpr>),
}

state_machine! {
    derive(Debug)
    repr_c(true)
    Reader(Start)

    // start of parse
    Start => {
        QuasiQuote => QuasiQuote
    },

    // `
    QuasiQuote => {
        QuasiQuote => QuasiQuote,
        Comma => Comma,
        Constant => Exit [ Form ],
        List => List,
        Symbol => Exit [ Quote ],
    },

    // `,
    Comma => {
        QuasiQuote => QuasiQuote,
        Comma => Comma,
        Constant => Exit [ Form ],
        EndList => Exit [ EndList ],
        List => CommaList,
        Symbol => Exit [ Form ],
    },

    // `(
    List => {
        QuasiQuote => QuasiQuote,
        Comma => CommaList,
        Constant => List [ Form ],
        Dot => List [ Dot ],
        EndList => Exit [ EndList ],
        List => List,
        Symbol => List [ Quote ],
    },

    // `,(
    CommaList => {
        At => CommaList [ At ],
        QuasiQuote => CommaList [ QuasiQuote ],
        Comma => CommaInList,
        Constant => CommaList [ Form ],
        EndList => Exit [ EndList ],
        List => List,
        Symbol => CommaList [ Quote ],
    },

    // `,(,
    CommaInList => {
        QuasiQuote => CommaList [ QuasiQuote ],
        Comma => CommaList,
        Constant => CommaList [ Form ],
        EndList => Exit [ EndList ],
        List => List,
        Symbol => CommaList [ Quote ],
    },
}

impl QqMachine {
    pub fn parse(mut source: String) -> Option<Vec<QqExpr>> {
        let mut expansion: Vec<QqExpr> = vec![];

        println!("parse: entry {}", source);

        if !source.starts_with('`') {
            return None;
        }

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

        let mut machine: StateMachine<Reader> = StateMachine::new();

        loop {
            match next_state() {
                None => {
                    println!("parse: error, unterminated expression.");
                    break;
                }
                Some((state, token)) => {
                    match machine.consume(&state) {
                        Err(_) => {
                            println!(
                                "parse: error on token {:?} in state {:?}",
                                token,
                                machine.state(),
                            );
                            
                            break;
                        },
                        Ok(output) => {
                            let new_state = machine.state();

                            println!("  ( {:?} {} ) enters {:?}", state, token, new_state);
                            match new_state {
                                ReaderState::QuasiQuote => {
                                    // Self::parse(source);
                                },
                                ReaderState::CommaList => match output {
                                    None => (),
                                    Some(qualifier) => {
                                        println!(
                                            "  ( {:?} [ {:?} ] {} ) enters {:?}",
                                            state, qualifier, token, new_state
                                        );
                                        match qualifier {
                                            ReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                            ReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                            _ => (),
                                        }
                                    }
                                },
                                ReaderState::Exit => {
                                    let qualifier = output.unwrap();
                                    println!(
                                        "  ( {:?} [ {:?} ] {} ) enters {:?}",
                                        state, qualifier, token, new_state
                                    );

                                    match qualifier {
                                        ReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                        ReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                        _ => (),
                                    }

                                    println!("parse: complete");
                                    break;
                                }
                                _ => {
                                }
                            }
                        }
                    }
                }
            }
        }

        Some(expansion)
    }
}
