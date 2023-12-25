#![allow(dead_code)]
    
use {
    rust_fsm::*,
    std::{cell::RefCell, result::Result},
};

pub struct QqReader {
    source: RefCell<String>,
    machine: RefCell<StateMachine<Reader>>,
}

#[derive(Debug)]
pub enum QqExpr {
    Dot(String),             // dotted list
    Form(String),            // plain form
    List(Vec<QqExpr>),       // list
    ListOf(Box<QqExpr>),     // make list of
    ListQuasi(Box<QqExpr>),  // make list of quasi
    Quote(String),           // quoted form
}

#[derive(Debug)]
pub enum QqState {
    Comma,
    CommaList,
    End,
    List,
    ListComma,
    Quasi,
}

//
// quasi quote exansion hierarchy
//
//    ` /
//        , /
//            basic
//            (
//        ( /
//            basic
//            `
//            (
//            , /
//                basic
//                (
//                @ /
//                   basic
//                   (
//

state_machine! {
    derive(Debug)
    repr_c(true)
    Reader(Quasi)

    // `
    Quasi => {
        Comma => Comma,              // ,
        Constant => End [ Form ],    // basic
        List => List,                // (
        Quasi => Quasi,              // `
        Symbol => End [ Quote ],     // basic
    },

    // `,
    Comma => {
        Constant => End [ Form ],    // ,basic
        List => CommaList,           // ,(
        Quasi => Quasi,              // ,`
        Symbol => End [ Form ],      // ,basic
    },

    // `,(
    CommaList => {
        Constant => List [ Form ],     // ,(basic
        EndList => End [ EndList ],    // ,()
        List => List,                  // ,((
        Quasi => CommaList [ Quasi ],  // ,(`
        Symbol => CommaList [ Quote ], // ,(basic
    },

    // `(
    List => {
        Comma => ListComma,          // (,
        Constant => List [ Form ],   // (basic
        EndList => End [ EndList ],  // ()
        List => List,                // ((
        Quasi => Quasi,              // (`
        Symbol => List [ Quote ],    // (basic
    },

    // `(,
    ListComma => {
        Comma => ListComma,               // ,,
        At => ListComma [ At ],           // ,@
        Constant => ListComma [ Form ],   // ,basic
        List => ListComma,                // ,(
        Quasi => Quasi,                   // ,`
        Symbol => ListComma [ Quote ],    // ,basic
    },
}

impl QqReader {
    pub fn new(mut source: String) -> Self {
        println!("reader: {}", source);
        let src = {
            if source.starts_with('`') {
                source.remove(0);
            }
            source
        };
        
        Self {
            source: RefCell::new(src),
            machine: RefCell::new(StateMachine::new()),
        }
    }

    fn read_char(&self) -> Option<char> {
        let mut src = self.source.borrow_mut();

        if src.is_empty() {
            None
        } else {
            Some(src.remove(0))
        }
    }

    fn unread_char(&self, ch: char) {
        let mut src = self.source.borrow_mut();

        src.insert(0, ch);
    }

    pub fn read(self) -> Result<String, String> {
        match self.parse() {
            Ok(vec) => Ok(Self::compile(vec)),
            Err(e) => Err(e),
        }
    }

    pub fn compile(list: Vec<QqExpr>) -> String {

        println!("compile: {:?}", list); 

        let mut out = "".to_string();

        for el in list {
            match el {
                QqExpr::Form(expr) => out.push_str(&format!(" {}", &expr)),
                QqExpr::Quote(expr) => out.push_str(&format!(" (:quote {})", &expr)),
                QqExpr::Dot(expr) => out.push_str(&format!(" . {}", &expr)),
                QqExpr::ListOf(expr) => out.push_str(&format!(" (mu:cons {:?} ())", &*expr)),
                QqExpr::List(vec) => {
                    for expr in vec {
                        out.push_str(" (mu:%cons");
                        match expr {
                            QqExpr::Form(expr) => out.push_str(&format!(" {}", &expr)),
                            QqExpr::Quote(expr) => out.push_str(&format!(" (:quote {})", &expr)),
                            QqExpr::Dot(expr) => out.push_str(&format!(" . {}", &expr)),
                            QqExpr::ListOf(expr) => out.push_str(&format!(" (mu:cons {:?} ())", &*expr)),
                            QqExpr::List(expr) => out.push_str(&Self::compile(expr)),
                            QqExpr::ListQuasi(expr) => out.push_str(&format!("(list `{:?})", &*expr)),
                        }
                    }
                    out.push_str(" ())")
                }
                QqExpr::ListQuasi(expr) => out.push_str(&format!("(list `{:?})", &*expr)),
            }
        }

        out
    }

    fn next_input_state(&self) -> Option<(ReaderInput, String)> {
        match self.read_char() {
            None => return None,
            Some(ch) => match ch {
                '(' => Some((ReaderInput::List, "(".to_string())),
                ')' => Some((ReaderInput::EndList, ")".to_string())),
                '`' => Some((ReaderInput::Quasi, "`".to_string())),
                ',' => Some((ReaderInput::Comma, ",".to_string())),
                '@' => Some((ReaderInput::At, "@".to_string())),
                _ => {
                    let mut token = String::from(ch);

                    loop {
                        match self.read_char() {
                            None => break,
                            Some(ch) => {
                                if ch.is_digit(10) || ch.is_alphabetic() {
                                    token.push(ch)
                                } else {
                                    self.unread_char(ch);
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
    }

    fn next(&self, state: &ReaderInput) -> Result<(Option<ReaderOutput>, QqState), QqState> {
        let mut machine = self.machine.borrow_mut();

        let consume = machine.consume(state);

        let qqstate = match machine.state() {
            ReaderState::Quasi => QqState::Quasi,
            ReaderState::Comma => QqState::Comma,
            ReaderState::CommaList => QqState::CommaList,
            ReaderState::ListComma => QqState::ListComma,
            ReaderState::List => QqState::List,
            ReaderState::End => QqState::End,
        };

        match consume {
            Err(_) => Err(qqstate),
            Ok(output) => Ok((output, qqstate)),
        }
    }

    pub fn parse(&self) -> Result<Vec<QqExpr>, String> {
        let mut expansion: Vec<QqExpr> = vec![];
                
        loop {
            match self.next_input_state() {
                None => break,
                Some((state, token)) => {
                    match self.next(&state) {
                        Err(qqstate) => {
                            return Err(format!(
                                "syntax: token {:?} in state {:?}",
                                token, qqstate,
                            ));
                        }
                        Ok((output, qqstate)) => {
                            match qqstate {
                                QqState::Quasi => {
                                    expansion.push(QqExpr::List(self.parse().unwrap()))
                                }
                                QqState::Comma => {}
                                QqState::CommaList => match output {
                                    None => expansion.push(QqExpr::List(self.parse().unwrap())),
                                    Some(qualifier) => match qualifier {
                                        ReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                        ReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                        ReaderOutput::EndList => break,
                                        _ => {
                                            return Err(
                                                "unimplemented CommaList element".to_string()
                                            )
                                        }
                                    },
                                },
                                QqState::List => match output {
                                    None => expansion.push(QqExpr::List(self.parse().unwrap())),
                                    Some(qualifier) => match qualifier {
                                        ReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                        ReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                        ReaderOutput::EndList => break,
                                        _ => return Err("unimplemented List element".to_string()),
                                    },
                                },
                                QqState::ListComma => match output {
                                    None => expansion.push(QqExpr::List(self.parse().unwrap())),
                                    Some(qualifier) => match qualifier {
                                        ReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                        ReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                        ReaderOutput::EndList => break,
                                        _ => {
                                            return Err(
                                                "unimplemented CommaList element".to_string()
                                            )
                                        }
                                    },
                                },
                                QqState::End => {
                                    match output.unwrap() {
                                        ReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                        ReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                        ReaderOutput::EndList => break,
                                        _ => return Err("unimplemented End element".to_string()),
                                    }
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
