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
    ListQuasi(Box<QqExpr>), // make list of quasi
    Quote(Tag),             // quoted form
}

#[derive(Debug)]
enum QqState {
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
    QReader(Quasi)

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
    pub fn new() -> Self {
        Self {
            machine: RefCell::new(StateMachine::new()),
        }
    }

    pub fn read(mu: &Mu, stream: Tag) -> exception::Result<Tag> {
        match Self::parse(mu, stream) {
            Ok(vec) => Ok(Self::compile(vec).unwrap()),
            Err(e) => Err(e),
        }
    }

    fn compile(_list: Vec<QqExpr>) -> exception::Result<Tag> {
        Ok(Tag::nil())
    }

    fn next_input_state(mu: &Mu, stream: Tag) -> exception::Result<Option<(QReaderInput, Tag)>> {
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
                            Type::Symbol => Ok(Some((QReaderInput::Symbol, form))),
                            _ => Ok(Some((QReaderInput::Constant, form))),
                        },
                    }
                }
            },
        }
    }

    fn next(mu: &Mu, state: &QReaderInput) -> exception::Result<(Option<QReaderOutput>, QqState)> {
        let mut machine = mu.qquote.machine.borrow_mut();

        match machine.consume(state) {
            Err(_) => panic!(),
            Ok(output) => {
                let qqstate = match *machine.state() {
                    QReaderState::Quasi => QqState::Quasi,
                    QReaderState::Comma => QqState::Comma,
                    QReaderState::CommaList => QqState::CommaList,
                    QReaderState::ListComma => QqState::ListComma,
                    QReaderState::List => QqState::List,
                    QReaderState::End => QqState::End,
                };

                Ok((Some(output.unwrap()), qqstate))
            }
        }
    }

    fn parse(mu: &Mu, stream: Tag) -> exception::Result<Vec<QqExpr>> {
        let mut expansion: Vec<QqExpr> = vec![];

        loop {
            match Self::next_input_state(mu, stream) {
                Err(e) => return Err(e),
                Ok(None) => break,
                Ok(Some((state, token))) => match Self::next(mu, &state) {
                    Err(_qqstate) => {
                        return Err(Exception::new(Condition::Syntax, "qquote", token));
                    }
                    Ok((output, qqstate)) => match qqstate {
                        QqState::Quasi => {
                            expansion.push(QqExpr::List(Self::parse(mu, stream).unwrap()))
                        }
                        QqState::Comma => {}
                        QqState::CommaList => match output {
                            None => expansion.push(QqExpr::List(Self::parse(mu, stream).unwrap())),
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
                            None => expansion.push(QqExpr::List(Self::parse(mu, stream).unwrap())),
                            Some(qualifier) => match qualifier {
                                QReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                QReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                QReaderOutput::EndList => break,
                                _ => panic!(),
                            },
                        },
                        QqState::ListComma => match output {
                            None => expansion.push(QqExpr::List(Self::parse(mu, stream).unwrap())),
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
                    },
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
