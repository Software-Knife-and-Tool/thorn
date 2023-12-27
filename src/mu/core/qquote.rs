//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! quasi quote reader
#![allow(dead_code)]
#[allow(unused_imports)]
use {
    crate::{
        core::{
            compile::Compiler,
            exception::{self, Condition, Exception},
            frame::Frame,
            funcall::Core as _,
            mu::{self, Core as _, Mu},
            reader::{Core as _, Reader},
            readtable::{map_char_syntax, SyntaxType},
            stream::{self, Core as _},
            types::{MuFunction as _, Tag, Type},
        },
        types::{
            char::Char,
            cons::{Cons, ConsIter, Core as _},
            fixnum::Fixnum,
            stream::{Core as _, Stream},
            streambuilder::StreamBuilder,
            vector::{Core as _, Vector},
        },
    },
    rust_fsm::*,
    std::cell::RefCell,
};

pub struct QqReader {
    machine: RefCell<StateMachine<QReader>>,
}

enum QqExpr {
    Dot(Tag),               // dotted list
    Form(Tag),              // plain form
    List(Vec<QqExpr>),      // list
    ListOf(Box<QqExpr>),    // make list of
    QuasiList(Box<QqExpr>), // quasi list
    Quote(Tag),             // quoted form
}

#[derive(Debug)]
enum QqState {
    QuasiComma,
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
//        basic
//        `
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
//                   `
//                   (
//

state_machine! {
    derive(Debug)
    repr_c(true)
    QReader(Quasi)

    // `
    Quasi => {
        Comma => QuasiComma,         // ,
        Constant => End [ Form ],    // basic
        List => List,                // (
        Quasi => Quasi,              // `
        Symbol => End [ Quote ],     // basic
    },

    // `,
    QuasiComma => {
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
    pub fn new() -> Self {
        Self {
            machine: RefCell::new(StateMachine::new()),
        }
    }

    pub fn read(mu: &Mu, stream: Tag) -> exception::Result<Tag> {
        match Self::parse(&Self::new(), mu, stream) {
            Ok(vec) => Ok(Self::compile(mu, vec).unwrap()),
            Err(e) => Err(e),
        }
    }

    fn print_annotated_tag(mu: &Mu, preface: &str, tag: Tag) {
        print!("{}: ", preface);
        mu.write(tag, true, mu.stdout).unwrap();
        println!()
    }

    fn compile(mu: &Mu, list: Vec<QqExpr>) -> exception::Result<Tag> {
        println!("compile: {}", list.len());
        for expr in list {
            match expr {
                QqExpr::Dot(tag) => Self::print_annotated_tag(mu, "    dot", tag),
                QqExpr::Form(tag) => Self::print_annotated_tag(mu, "    form", tag),
                QqExpr::List(_vec) => Self::print_annotated_tag(mu, "    list", Tag::nil()),
                QqExpr::ListOf(_box_tag) => {
                    Self::print_annotated_tag(mu, "    list-of", Tag::nil())
                }
                QqExpr::QuasiList(_box_tag) => {
                    Self::print_annotated_tag(mu, "    quasi-list", Tag::nil())
                }
                QqExpr::Quote(tag) => Self::print_annotated_tag(mu, "    quote", tag),
            }
        }

        Ok(Tag::nil())
    }

    fn next_input_state(mu: &Mu, stream: Tag) -> exception::Result<Option<(QReaderInput, Tag)>> {
        Reader::read_ws(mu, stream).unwrap();
        match Stream::read_char(mu, stream) {
            Err(e) => Err(e),
            Ok(None) => return Ok(None),
            Ok(Some(ch)) => match ch {
                '(' => Ok(Some((QReaderInput::List, Char::as_tag('(')))),
                ')' => Ok(Some((QReaderInput::EndList, Char::as_tag(')')))),
                '`' => Ok(Some((QReaderInput::Quasi, Char::as_tag('`')))),
                ',' => Ok(Some((QReaderInput::Comma, Char::as_tag(',')))),
                '@' => Ok(Some((QReaderInput::At, Char::as_tag('@')))),
                _ => {
                    Stream::unread_char(mu, stream, ch).unwrap();

                    match <mu::Mu as stream::Core>::read(mu, stream, false, Tag::nil(), false) {
                        Err(e) => return Err(e),
                        Ok(form) => match Tag::type_of(form) {
                            Type::Cons => panic!(),
                            Type::Symbol => Ok(Some((QReaderInput::Symbol, form))),
                            _ => Ok(Some((QReaderInput::Constant, form))),
                        },
                    }
                }
            },
        }
    }

    fn next(&self, state: &QReaderInput) -> exception::Result<(Option<QReaderOutput>, QqState)> {
        let mut machine = self.machine.borrow_mut();

        match machine.consume(state) {
            Err(_) => panic!(),
            Ok(output) => {
                let qqstate = match machine.state() {
                    QReaderState::Quasi => QqState::Quasi,
                    QReaderState::QuasiComma => QqState::QuasiComma,
                    QReaderState::CommaList => QqState::CommaList,
                    QReaderState::ListComma => QqState::ListComma,
                    QReaderState::List => QqState::List,
                    QReaderState::End => QqState::End,
                };

                match output {
                    None => Ok((None, qqstate)),
                    Some(output) => Ok((Some(output), qqstate)),
                }
            }
        }
    }

    fn parse(&self, mu: &Mu, stream: Tag) -> exception::Result<Vec<QqExpr>> {
        let mut expansion: Vec<QqExpr> = vec![];

        loop {
            match Self::next_input_state(mu, stream) {
                Err(e) => return Err(e),
                Ok(None) => break,
                Ok(Some((state, token))) => match self.next(&state) {
                    Err(_qqstate) => {
                        return Err(Exception::new(Condition::Syntax, "qquote", token));
                    }
                    Ok((output, qqstate)) => {
                        // println!("qqstate: {:?}", qqstate);
                        match qqstate {
                            QqState::Quasi => {
                                println!("quasi:");

                                expansion.push(QqExpr::QuasiList(Box::new(QqExpr::List(
                                    Self::parse(&Self::new(), mu, stream).unwrap(),
                                ))))
                            }
                            QqState::QuasiComma => {}
                            QqState::CommaList => match output {
                                None => {
                                    expansion.push(QqExpr::List(self.parse(mu, stream).unwrap()))
                                }
                                Some(qualifier) => match qualifier {
                                    QReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                    QReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                    QReaderOutput::EndList => break,
                                    _ => {
                                        panic!()
                                    }
                                },
                            },
                            QqState::List => match output {
                                None => {
                                    println!("list:");
                                    expansion.push(QqExpr::List(self.parse(mu, stream).unwrap()))
                                }
                                Some(qualifier) => match qualifier {
                                    QReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                    QReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                    QReaderOutput::EndList => break,
                                    _ => panic!(),
                                },
                            },
                            QqState::ListComma => match output {
                                None => {
                                    expansion.push(QqExpr::List(self.parse(mu, stream).unwrap()))
                                }
                                Some(qualifier) => match qualifier {
                                    QReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                    QReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                    QReaderOutput::EndList => break,
                                    _ => panic!(),
                                },
                            },
                            QqState::End => match output.unwrap() {
                                QReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                QReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                QReaderOutput::EndList => break,
                                _ => panic!(),
                            },
                        }
                    }
                },
            }
        }

        Ok(expansion)
    }
}

pub trait MuFunction {
    fn mu_qquote(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for QqReader {
    fn mu_qquote(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match mu.fp_argv_check("%qquote".to_string(), &[Type::Stream], fp) {
            Ok(_) => match Self::read(mu, fp.argv[0]) {
                Ok(tag) => tag,
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        };

        Ok(())
    }
}
