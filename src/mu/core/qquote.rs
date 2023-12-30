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
    Comma,               // comma form
    Dot(Tag),            // dotted list
    Form(Tag),           // plain form
    List(Vec<QqExpr>),   // list
    ListOf(Box<QqExpr>), // make list of
    Quasi(Box<QqExpr>),  // quasi list
    Quote(Tag),          // quoted form
}

#[derive(Debug)]
enum QqState {
    Start,
    Quasi,
    Comma,
    List,
}

//
// quasiquote expansion hierarchy
//
//    `:
//      basic
//      `
//      ,:
//        basic
//        `
//        (
//      (:
//        basic
//        `
//        (
//        ,:
//          basic
//          `
//          (
//          @:
//            basic
//            `
//            (
//

state_machine! {
    derive(Debug)
    repr_c(true)
    QReader(Start)

    Start => {
        Quasi => Quasi,
    },

    // `
    Quasi => {
        Comma => Quasi [ Comma ],        // ,
        Constant => Quasi [ Form ],      // basic
        List => List,                    // (
        Quasi => Quasi [ New ],          // `
        Symbol => Quasi [ Quote ],       // basic
    },

    // `(
    List => {
        Comma => Comma,                  // ,
        Constant => List [ Form ],       // basic
        Dot => List [ Dot ],             // .
        EndList => List [ End ],         // ()
        List => List [ Form ],           // (
        Quasi => List [ Quasi ],         // `
        Symbol => List [ Quote ],        // basic
    },

    // `(,
    Comma => {
        Comma => Comma,                  // ,
        At => Comma [ At ],              // @
        Constant => Comma [ Form ],      // basic
        List => Comma [ Form ],          // (
        Quasi => Quasi [ New ],          // `
        Symbol => Comma [ Quote ],       // basic
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

    fn compile(mu: &Mu, expr: QqExpr) -> exception::Result<Tag> {
        println!("compile:");

        Self::print_indent(mu, 1, expr);
        Ok(Tag::nil())
    }

    fn print_state(&self) {
        let machine = self.machine.borrow();

        println!("current state: {:?}", machine.state());
    }

    fn print_annotated_tag(mu: &Mu, preface: &str, tag: Tag) {
        print!("{}: ", preface);
        mu.write(tag, true, mu.stdout).unwrap();
        println!()
    }

    fn print_indent(mu: &Mu, indent: i32, expr: QqExpr) {
        for _ in 1..indent * 2 {
            print!(" ");
        }
        match expr {
            QqExpr::Comma => println!("QqExpr::Comma"),
            QqExpr::Dot(tag) => Self::print_annotated_tag(mu, "dot", tag),
            QqExpr::Form(tag) => Self::print_annotated_tag(mu, "form", tag),
            QqExpr::List(vec) => {
                println!("list: {}", vec.len());
                for expr in vec {
                    Self::print_indent(mu, indent + 1, expr);
                }
            }
            QqExpr::ListOf(_box_tag) => Self::print_annotated_tag(mu, "list-of", Tag::nil()),
            QqExpr::Quasi(boxed_expr) => {
                println!("quasi:");
                Self::print_indent(mu, indent + 1, *boxed_expr);
            }
            QqExpr::Quote(tag) => Self::print_annotated_tag(mu, "quote", tag),
        }
    }

    fn next_input_state(&self, mu: &Mu, stream: Tag) -> exception::Result<Option<(QReaderInput, Tag)>> {

        Reader::read_ws(mu, stream).unwrap();
        match Stream::read_char(mu, stream) {
            Err(e) => Err(e),
            Ok(None) => Ok(None),
            Ok(Some(ch)) => match ch {
                '(' => Ok(Some((QReaderInput::List, Char::as_tag('(')))),
                ')' => Ok(Some((QReaderInput::EndList, Char::as_tag(')')))),
                ',' => Ok(Some((QReaderInput::Comma, Char::as_tag(',')))),
                '@' => Ok(Some((QReaderInput::At, Char::as_tag('@')))),
                '`' => Ok(Some((QReaderInput::Quasi, Char::as_tag('`')))),
                _ => {
                    Stream::unread_char(mu, stream, ch).unwrap();

                    match <mu::Mu as stream::Core>::read(mu, stream, false, Tag::nil(), false) {
                        Err(e) => Err(e),
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
            Err(_) => {
                Err(Exception::new(Condition::Syntax,
                                   "illegal token: type",
                                   Tag::nil()))
            },
            Ok(output) => {
                let qqstate = match machine.state() {
                    QReaderState::Start => QqState::Start,
                    QReaderState::Quasi => QqState::Quasi,
                    QReaderState::Comma => QqState::Comma,
                    QReaderState::List => QqState::List,
                };

                match output {
                    None => Ok((None, qqstate)),
                    Some(output) => Ok((Some(output), qqstate)),
                }
            }
        }
    }

    fn parse(&self, mu: &Mu, stream: Tag) -> exception::Result<QqExpr> {
        let mut expansion: Vec<QqExpr> = vec![];

        loop {
            match Self::next_input_state(self, mu, stream) {
                Err(e) => return Err(e),
                Ok(None) => return Err(Exception::new(Condition::Syntax, "incomplete expression", Tag::nil())),
                Ok(Some((state, token))) => {
                    // Self::print_state(self);
                    // print!("next reader state {:?}", state);
                    // Self::print_annotated_tag(mu, " ", token);
                    match self.next(&state) {
                        Err(e) => return Err(e),
                        Ok((output, qqstate)) => match qqstate {
                            QqState::Start => {},
                            QqState::Quasi => match output {
                                None => {},
                                Some(qualifier) => match qualifier {
                                    QReaderOutput::New => expansion.push(QqExpr::Quasi(Box::new(
                                        Self::parse(&Self::new(), mu, stream).unwrap(),
                                    ))),
                                    QReaderOutput::Comma => expansion.push(QqExpr::Comma),
                                    QReaderOutput::Form => {
                                        expansion.push(QqExpr::Form(token));
                                        return Ok(QqExpr::List(expansion))
                                    },
                                    QReaderOutput::Quote => return Ok(QqExpr::Quote(token)),
                                    _ => {
                                        panic!()
                                    }
                                },
                            },
                            QqState::Comma => match output {
                                None => expansion.push(QqExpr::Comma),
                                Some(qualifier) => match qualifier {
                                    QReaderOutput::At => return Ok(QqExpr::List(expansion)),
                                    QReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                    QReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                    _ => {
                                        panic!()
                                    }
                                },
                            },
                            QqState::List => match output {
                                None => return Ok(self.parse(mu, stream).unwrap()),
                                Some(qualifier) => match qualifier {
                                    QReaderOutput::Dot => expansion.push(QqExpr::Quote(token)),
                                    QReaderOutput::End => return Ok(QqExpr::List(expansion)),
                                    QReaderOutput::Form => expansion.push(QqExpr::Form(token)),
                                    QReaderOutput::Quasi => return Ok(QqExpr::List(expansion)),
                                    QReaderOutput::Quote => expansion.push(QqExpr::Quote(token)),
                                    _ => panic!(),
                                },
                            },
                        },
                    }
                }
            }
        }
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
