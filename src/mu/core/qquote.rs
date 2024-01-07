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
            reader::{Core as _, Reader},
            stream::{self, Core as _},
            types::{Tag, Type},
        },
        types::{
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

enum QqForm {
    Constant(Tag),
    Cons(Tag),
    Nil(Tag),
    Symbol(Tag),
}

#[derive(Debug)]
enum QqState {
    Start,
    Quasi,
    QuasiComma,
    QuasiList,
    QuasiListComma,
}

//
// quasiquote expansion hierarchy
//
//    `:
//      form
//      `
//      ,:
//        form
//        `
//        (
//      (:
//        form
//        `
//        (
//        ,:
//          form
//          `
//          (
//          @:
//            form
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

    Quasi => {
        Comma => QuasiComma,                 // ,
        List => QuasiList,                   // (
        Form => Quasi [ Form ],              // form
        Quasi => Quasi [ Form ],             // `
    },

    QuasiComma => {
        Comma => QuasiComma [ Error ],       // comma not in qquote
        List => QuasiComma [ List ],         // (
        EndList => QuasiComma [ End ],       //
        Form => QuasiComma [ Form ],         // form
        Quasi => QuasiComma [ Quasi ],       // `
    },

    QuasiList => {
        Comma => QuasiListComma,             // ,
        Form => QuasiList [ Form ],      // form
        Dot => QuasiList [ Dot ],            // .
        EndList => QuasiList [ End ],        // )
        List => QuasiList [ ListOfQuasi ],   // (
        Quasi => QuasiList [ ListOfQuasi ],  // `
    },

    QuasiListComma => {
        Comma => QuasiListComma,             // ,
        At => QuasiListComma [ At ],         // ,@
        Form => QuasiList [ Form ],          // form
        List => QuasiList [ List ],          // ,(
        Quasi => QuasiList [ New ],          // ,`
    },
}

impl QqReader {
    pub fn new(mu: &Mu) -> Self {
        Self {
            machine: RefCell::new(StateMachine::new()),
            append_sym: mu.append_,
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
                    match expr {
                        QqExpr::Dot(_) => panic!(),
                        _ => {
                            append_chain = Cons::new(
                                self.append_sym,
                                Cons::new(
                                    Self::compile(self, mu, expr).unwrap(),
                                    Cons::new(append_chain, Tag::nil()).evict(mu),
                                )
                                .evict(mu),
                            )
                            .evict(mu)
                        }
                    }
                }

                Ok(append_chain)
            }
            QqExpr::Quasi(expr) => Ok(Self::compile(self, mu, *expr).unwrap()),
            QqExpr::Dot(_) => Err(Exception::new(
                Condition::Syntax,
                "qquote",
                Symbol::keyword("dot"),
            )),
        }
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

    fn read_syntax(&self, mu: &Mu, stream: Tag) -> exception::Result<Option<QReaderInput>> {
        Reader::read_ws(mu, stream).unwrap();
        match Stream::read_char(mu, stream) {
            Err(e) => Err(e),
            Ok(None) => Ok(None),
            Ok(Some(ch)) => match ch {
                '(' => Ok(Some(QReaderInput::List)),
                ')' => Ok(Some(QReaderInput::EndList)),
                ',' => Ok(Some(QReaderInput::Comma)),
                '@' => Ok(Some(QReaderInput::At)),
                '`' => Ok(Some(QReaderInput::Quasi)),
                _ => {
                    Stream::unread_char(mu, stream, ch).unwrap();
                    Ok(Some(QReaderInput::Form))
                }
            },
        }
    }

    fn read_form(mu: &Mu, stream: Tag) -> exception::Result<QqForm> {
        match <mu::Mu as stream::Core>::read(mu, stream, false, Tag::nil(), false) {
            Err(e) => Err(e),
            Ok(form) => match form.type_of() {
                Type::Cons => Ok(QqForm::Cons(form)),
                Type::Symbol => Ok(QqForm::Symbol(form)),
                _ => Ok(QqForm::Constant(form)),
            },
        }
    }

    fn next(&self, state: &QReaderInput) -> exception::Result<(Option<QReaderOutput>, QqState)> {
        let mut machine = self.machine.borrow_mut();

        match machine.consume(state) {
            Err(_) => Err(Exception::new(
                Condition::Syntax,
                "qquote",
                Symbol::keyword("next"),
            )),
            Ok(output) => {
                let qqstate = match machine.state() {
                    QReaderState::Start => QqState::Start,
                    QReaderState::Quasi => QqState::Quasi,
                    QReaderState::QuasiComma => QqState::QuasiComma,
                    QReaderState::QuasiList => QqState::QuasiList,
                    QReaderState::QuasiListComma => QqState::QuasiListComma,
                };

                match output {
                    None => Ok((None, qqstate)),
                    Some(output) => Ok((Some(output), qqstate)),
                }
            }
        }
    }

    /*
    QReaderOutput::New => match self.parse(mu, stream) {
        Err(e) => return Err(e),
        Ok(expr) => {
            expansion.push(QqExpr::Quasi(Box::new(expr)))
        }
    },
     */

    fn parse(&self, mu: &Mu, stream: Tag) -> exception::Result<QqExpr> {
        let mut expansion: Vec<QqExpr> = vec![];

        loop {
            // Self::print_state(self);
            match Self::read_syntax(self, mu, stream) {
                Err(e) => return Err(e),
                Ok(None) => {
                    return Err(Exception::new(
                        Condition::Syntax,
                        "qquote",
                        Symbol::keyword("input"),
                    ))
                }
                Ok(Some(state)) => {
                    // Self::print_annotated_tag(mu, &format!("reader state {:?}", state), token);
                    match self.next(&state) {
                        Err(e) => return Err(e),
                        Ok((output, next_state)) => {
                            // println!("next_state {:?} output {:?}", next_state, output);
                            match next_state {
                                QqState::Start => {}
                                QqState::Quasi => match output {
                                    None => {}
                                    Some(qualifier) => match qualifier {
                                        QReaderOutput::Form => match Self::read_form(mu, stream) {
                                            Err(e) => return Err(e),
                                            Ok(qqform) => match qqform {
                                                QqForm::Symbol(form) | QqForm::Constant(form) => {
                                                    return Ok(QqExpr::Form(form))
                                                }
                                                _ => panic!(),
                                            },
                                        },
                                        _ => panic!("{:?}", qualifier),
                                    },
                                },
                                QqState::QuasiComma => match output {
                                    None => {}
                                    Some(qualifier) => match qualifier {
                                        QReaderOutput::Form => match Self::read_form(mu, stream) {
                                            Err(e) => return Err(e),
                                            Ok(qqform) => match qqform {
                                                QqForm::Symbol(form) | QqForm::Constant(form) => {
                                                    return Ok(QqExpr::Form(form))
                                                }
                                                _ => panic!(),
                                            },
                                        },
                                        QReaderOutput::End => match Self::read_form(mu, stream) {
                                            Err(e) => return Err(e),
                                            Ok(qqform) => match qqform {
                                                QqForm::Symbol(form) | QqForm::Constant(form) => {
                                                    return Ok(QqExpr::Form(form))
                                                }
                                                _ => panic!(),
                                            },
                                        },
                                        QReaderOutput::Quasi => match Self::read_form(mu, stream) {
                                            Err(e) => return Err(e),
                                            Ok(qqform) => match qqform {
                                                QqForm::Symbol(form)
                                                | QqForm::Cons(form)
                                                | QqForm::Nil(form)
                                                | QqForm::Constant(form) => {
                                                    return Ok(QqExpr::Form(form))
                                                }
                                            },
                                        },
                                        QReaderOutput::List => {
                                            Stream::unread_char(mu, stream, '(').unwrap();
                                            return Ok(QqExpr::Form(
                                                <mu::Mu as stream::Core>::read(
                                                    mu,
                                                    stream,
                                                    false,
                                                    Tag::nil(),
                                                    false,
                                                )
                                                .unwrap(),
                                            ));
                                        }
                                        QReaderOutput::Error => {
                                            return Err(Exception::new(
                                                Condition::Syntax,
                                                "qquote",
                                                Symbol::keyword("error"),
                                            ))
                                        }
                                        _ => panic!("{:?}", qualifier),
                                    },
                                },
                                QqState::QuasiListComma => match output {
                                    None => match self.parse(mu, stream) {
                                        Err(e) => return Err(e),
                                        Ok(expr) => return Ok(QqExpr::Quasi(Box::new(expr))),
                                    },
                                    Some(qualifier) => match qualifier {
                                        QReaderOutput::At => return Ok(QqExpr::List(expansion)),
                                        QReaderOutput::Form => match Self::read_form(mu, stream) {
                                            Err(e) => return Err(e),
                                            Ok(qqform) => match qqform {
                                                QqForm::Symbol(form) | QqForm::Constant(form) => {
                                                    expansion.push(QqExpr::Form(form))
                                                }
                                                _ => panic!(),
                                            },
                                        },
                                        _ => panic!("{:?}", qualifier),
                                    },
                                },
                                QqState::QuasiList => match output {
                                    None => match self.parse(mu, stream) {
                                        Err(e) => return Err(e),
                                        Ok(expr) => return Ok(QqExpr::List(vec![expr])),
                                    },
                                    Some(qualifier) => match qualifier {
                                        QReaderOutput::End => return Ok(QqExpr::List(expansion)),
                                        QReaderOutput::Form => match Self::read_form(mu, stream) {
                                            Err(e) => return Err(e),
                                            Ok(qqform) => match qqform {
                                                QqForm::Constant(form) => {
                                                    expansion.push(QqExpr::ListOf(form))
                                                }
                                                QqForm::Symbol(symbol) => {
                                                    expansion.push(QqExpr::ListOf(Cons::vlist(
                                                        mu,
                                                        &[Symbol::keyword("quote"), symbol],
                                                    )))
                                                }
                                                _ => panic!(),
                                            },
                                        },
                                        QReaderOutput::Dot => {}
                                        QReaderOutput::List => {
                                            Stream::unread_char(mu, stream, '(').unwrap();
                                            match Self::read_form(mu, stream) {
                                                Err(e) => return Err(e),
                                                Ok(qqform) => match qqform {
                                                    QqForm::Cons(form) => {
                                                        expansion.push(QqExpr::ListOf(form))
                                                    }
                                                    _ => panic!(),
                                                },
                                            }
                                        }
                                        QReaderOutput::ListOfQuasi => {
                                            Stream::unread_char(mu, stream, '`').unwrap();

                                            match Self::read(mu, stream) {
                                                Err(e) => return Err(e),
                                                Ok(form) => return Ok(QqExpr::ListOf(form)),
                                            };
                                        }
                                        _ => panic!("{:?}", qualifier),
                                    },
                                },
                            }
                        }
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
        fp.value = match mu.fp_argv_check("%qquote", &[Type::Stream], fp) {
            Ok(_) => match Self::read(mu, fp.argv[0]) {
                Ok(tag) => tag,
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        };

        Ok(())
    }
}
