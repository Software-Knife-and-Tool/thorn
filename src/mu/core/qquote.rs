//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! quasiquote reader
use {
    crate::{
        core::{
            exception::{self, Condition, Exception},
            frame::Frame,
            funcall::Core as _,
            mu::{self, Mu},
            namespace::Core,
            reader::{Core as _, Reader},
            stream::{self, Core as _},
            types::{Tag, Type},
        },
        types::{
            char::Char,
            cons::{Cons, Core as _},
            stream::{Core as _, Stream},
            symbol::{Core as _, Symbol},
        },
    },
    rust_fsm::*,
    std::cell::RefCell,
};

pub struct QqReader {
    machine: RefCell<StateMachine<QReader>>,
    append_sym: Tag,
}

enum QqExpr {
    Comma(Box<QqExpr>), // comma form
    Dot(Tag),           // dotted list
    Form(Tag),          // plain form
    List(Vec<QqExpr>),  // list
    ListOf(Tag),        // make list of
    Quasi(Box<QqExpr>), // quasi list
    Quote(Tag),         // quoted form
}

#[derive(Debug)]
enum QqState {
    Start,
    Quasi,
    QuasiComma,
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
        Comma => QuasiComma,             // ,
        Constant => Quasi [ Form ],      // basic
        Symbol => Quasi [ Quote ],       // basic
        List => List,                    // (
        Quasi => Quasi [ New ],          // `
    },

    QuasiComma => {
        Constant => QuasiComma [ Form ], // basic
        Symbol => QuasiComma [ Form ],   // basic
        List => QuasiComma [  List ],    // (
        Quasi => QuasiComma [ Quasi ],   // `
    },

    List => {
        Comma => Comma,                  // ,
        Constant => List [ ListOf ],     // basic
        Symbol => List [ Quote ],        // basic
        Dot => List [ Dot ],             // .
        EndList => List [ End ],         // ()
        List => List [ Form ],           // (
        Quasi => List [ Quasi ],         // `
    },

    Comma => {
        Comma => Comma,                  // ,
        At => Comma [ At ],              // ,@
        Constant => List [ Form ],       // ,basic
        Symbol => Comma [ Quote ],       // ,basic
        List => Comma [ Form ],          // ,(
        Quasi => Quasi [ New ],          // ,`
    },
}

impl QqReader {
    pub fn new(mu: &Mu) -> Self {
        Self {
            machine: RefCell::new(StateMachine::new()),
            append_sym: <Mu as Core>::intern_symbol(
                mu,
                mu.mu_ns,
                "%append".to_string(),
                Tag::nil(),
            ),
        }
    }

    pub fn read(mu: &Mu, stream: Tag) -> exception::Result<Tag> {
        let parser = Self::new(mu);
        match Self::parse(&parser, mu, stream) {
            Ok(vec) => Ok(Self::compile(&parser, mu, vec).unwrap()),
            Err(e) => Err(e),
        }
    }

    fn compile(&self, mu: &Mu, expr: QqExpr) -> exception::Result<Tag> {
        match expr {
            QqExpr::Form(tag) => Ok(tag),
            QqExpr::Comma(boxed_expr) => Self::compile(self, mu, *boxed_expr),
            QqExpr::ListOf(tag) => {
                let vlist = vec![Symbol::keyword("quote"), Cons::vlist(mu, &[tag])];

                Ok(Cons::vlist(mu, &vlist))
            }
            QqExpr::Quote(tag) => {
                let vlist = vec![Symbol::keyword("quote"), tag];

                Ok(Cons::vlist(mu, &vlist))
            }
            QqExpr::List(vec) => {
                let mut append_chain = Tag::nil();

                for expr in vec.into_iter().rev() {
                    append_chain = Cons::new(
                        self.append_sym,
                        Cons::new(
                            Self::compile(self, mu, expr).unwrap(),
                            Cons::new(append_chain, Tag::nil()).evict(mu),
                        )
                        .evict(mu),
                    )
                    .evict(mu);
                }

                Ok(append_chain)
            }
            _ => Ok(Tag::nil()),
        }

        /*
                QqExpr::Dot(tag) => Self::print_annotated_tag(mu, "QqExpr::Dot:", tag),

                QqExpr::Quasi(boxed_expr) => {
                    println!("QqExpr::Quasi:");
                    Self::print_indent(mu, indent + 1, *boxed_expr);
                }

        }
            */
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
            QqExpr::Comma(boxed_expr) => {
                println!("QqExpr::Comma: ");
                Self::print_indent(mu, 0, *boxed_expr);
            }
            QqExpr::Dot(tag) => Self::print_annotated_tag(mu, "QqExpr::Dot:", tag),
            QqExpr::Form(tag) => Self::print_annotated_tag(mu, "QqExpr::Form:", tag),
            QqExpr::List(vec) => {
                println!("QqExpr::List: {}", vec.len());
                for expr in vec {
                    Self::print_indent(mu, indent + 1, expr);
                }
            }
            QqExpr::ListOf(_box_tag) => Self::print_annotated_tag(mu, "QqExpr::ListOf", Tag::nil()),
            QqExpr::Quasi(boxed_expr) => {
                println!("QqExpr::Quasi:");
                Self::print_indent(mu, indent + 1, *boxed_expr);
            }
            QqExpr::Quote(tag) => Self::print_annotated_tag(mu, "QqExpr::Quote", tag),
        }
    }

    fn next_input_state(
        &self,
        mu: &Mu,
        stream: Tag,
    ) -> exception::Result<Option<(QReaderInput, Tag)>> {
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
            Err(_) => Err(Exception::new(
                Condition::Syntax,
                "illegal token: type",
                Tag::nil(),
            )),
            Ok(output) => {
                let qqstate = match machine.state() {
                    QReaderState::Start => QqState::Start,
                    QReaderState::Quasi => QqState::Quasi,
                    QReaderState::QuasiComma => QqState::QuasiComma,
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
            // Self::print_state(self);
            match Self::next_input_state(self, mu, stream) {
                Err(e) => return Err(e),
                Ok(None) => {
                    return Err(Exception::new(
                        Condition::Syntax,
                        "incomplete expression",
                        Tag::nil(),
                    ))
                }
                Ok(Some((state, token))) => {
                    /*
                    Self::print_annotated_tag(mu,
                                              &format!("reader state {:?}", state),
                    token);
                    */
                    match self.next(&state) {
                        Err(e) => return Err(e),
                        Ok((output, qqstate)) => match qqstate {
                            QqState::Start => {}
                            QqState::Quasi => match output {
                                None => {}
                                Some(qualifier) => match qualifier {
                                    QReaderOutput::New => expansion.push(QqExpr::Quasi(Box::new(
                                        self.parse(mu, stream).unwrap(),
                                    ))),
                                    QReaderOutput::Form => return Ok(QqExpr::Form(token)),
                                    QReaderOutput::Quote => return Ok(QqExpr::Quote(token)),
                                    _ => {
                                        panic!()
                                    }
                                },
                            },
                            QqState::QuasiComma => match output {
                                None => {}
                                Some(qualifier) => match qualifier {
                                    QReaderOutput::Form => return Ok(QqExpr::Form(token)),
                                    QReaderOutput::List => return Ok(QqExpr::Form(token)),
                                    QReaderOutput::Quasi => return Ok(QqExpr::Form(token)),
                                    _ => {
                                        panic!()
                                    }
                                },
                            },
                            QqState::Comma => match output {
                                None => {
                                    return Ok(QqExpr::Comma(Box::new(
                                        self.parse(mu, stream).unwrap(),
                                    )))
                                }
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
                                    QReaderOutput::ListOf => expansion.push(QqExpr::ListOf(token)),
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
