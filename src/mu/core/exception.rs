//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu exceptions:
//!    Condition
//!    Exception
//!    `Result<Exception>`
use {
    crate::{
        core::{
            frame::Frame,
            mu::{Core as _, Mu},
            types::{Tag, Type},
        },
        types::symbol::{Core as _, Symbol},
    },
    std::fmt,
};

#[cfg(feature = "async")]
use futures::executor::block_on;

pub type Result<T> = std::result::Result<T, Exception>;

#[derive(Clone)]
pub struct Exception {
    pub object: Tag,
    pub condition: Condition,
    pub source: Tag,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Condition {
    Arity,
    Except,
    Eof,
    Error,
    Open,
    Namespace,
    Range,
    Read,
    Stream,
    Syntax,
    Type,
    Unbound,
    Write,
    ZeroDivide,
}

lazy_static! {
    static ref CONDMAP: Vec<(Tag, Condition)> = vec![
        (Symbol::keyword("arity"), Condition::Arity),
        (Symbol::keyword("div0"), Condition::ZeroDivide),
        (Symbol::keyword("except"), Condition::Except),
        (Symbol::keyword("eof"), Condition::Eof),
        (Symbol::keyword("error"), Condition::Error),
        (Symbol::keyword("open"), Condition::Open),
        (Symbol::keyword("ns"), Condition::Namespace),
        (Symbol::keyword("range"), Condition::Range),
        (Symbol::keyword("read"), Condition::Read),
        (Symbol::keyword("stream"), Condition::Stream),
        (Symbol::keyword("syntax"), Condition::Syntax),
        (Symbol::keyword("type"), Condition::Type),
        (Symbol::keyword("unbound"), Condition::Unbound),
        (Symbol::keyword("write"), Condition::Write),
    ];
}

impl fmt::Debug for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}:{}", self.condition, self.source)
    }
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}:{}", self.condition, self.source)
    }
}

impl Exception {
    pub fn new(condition: Condition, src: &str, object: Tag) -> Self {
        Exception {
            object,
            condition,
            source: Symbol::keyword(src),
        }
    }

    fn map_condition(keyword: Tag) -> Result<Condition> {
        #[allow(clippy::unnecessary_to_owned)]
        let condmap = CONDMAP
            .to_vec()
            .into_iter()
            .find(|cond| keyword.eq_(cond.0));

        match condmap {
            Some(entry) => Ok(entry.1),
            _ => Err(Exception::new(
                Condition::Syntax,
                "exception::map_condition",
                keyword,
            )),
        }
    }

    fn map_condkey(cond: Condition) -> Result<Tag> {
        #[allow(clippy::unnecessary_to_owned)]
        let condmap = CONDMAP
            .to_vec()
            .into_iter()
            .find(|condtab| cond == condtab.1);

        match condmap {
            Some(entry) => Ok(entry.0),
            _ => panic!(),
        }
    }
}

pub trait MuFunction {
    fn mu_with_ex(mu: &Mu, fp: &mut Frame) -> Result<()>;
    fn mu_raise(mu: &Mu, fp: &mut Frame) -> Result<()>;
}

impl MuFunction for Exception {
    fn mu_raise(_: &Mu, fp: &mut Frame) -> Result<()> {
        let src = fp.argv[0];
        let condition = fp.argv[1];

        match Tag::type_of(condition) {
            Type::Keyword => match Self::map_condition(condition) {
                Ok(cond) => Err(Self::new(cond, "raise", src)),
                Err(_) => Err(Self::new(Condition::Type, "raise", condition)),
            },
            _ => Err(Self::new(Condition::Type, "raise", condition)),
        }
    }

    fn mu_with_ex(mu: &Mu, fp: &mut Frame) -> Result<()> {
        let handler = fp.argv[0];
        let thunk = fp.argv[1];

        fp.value = match Tag::type_of(thunk) {
            Type::Function => match Tag::type_of(handler) {
                Type::Function => {
                    {
                        #[cfg(feature = "async")]
                        let dynamic_ref = block_on(mu.dynamic.read());
                        #[cfg(not(feature = "async"))]
                        let dynamic_ref = mu.dynamic.borrow();

                        #[cfg(feature = "async")]
                        let mut unwind_ref = block_on(mu.unwind.write());
                        #[cfg(not(feature = "async"))]
                        let mut unwind_ref = mu.unwind.borrow_mut();

                        unwind_ref.push(dynamic_ref.len())
                    }

                    match mu.apply(thunk, Tag::nil()) {
                        Ok(value) => value,
                        Err(e) => {
                            let args =
                                vec![e.object, Self::map_condkey(e.condition).unwrap(), e.source];
                            match mu.apply_(handler, args) {
                                Ok(value) => {
                                    #[cfg(feature = "async")]
                                    let mut dynamic_ref = block_on(mu.dynamic.write());
                                    #[cfg(not(feature = "async"))]
                                    let mut dynamic_ref = mu.dynamic.borrow_mut();

                                    #[cfg(feature = "async")]
                                    let mut unwind_ref = block_on(mu.unwind.write());
                                    #[cfg(not(feature = "async"))]
                                    let mut unwind_ref = mu.unwind.borrow_mut();

                                    match unwind_ref.pop() {
                                        Some(len) => {
                                            dynamic_ref.resize(len, (0, 0));
                                            value
                                        }
                                        None => panic!("dynamic stack underflow"),
                                    }
                                }
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
                _ => return Err(Exception::new(Condition::Type, "with-ex", handler)),
            },
            _ => return Err(Exception::new(Condition::Type, "with-ex", thunk)),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn exception() {
        assert!(true)
    }
}
