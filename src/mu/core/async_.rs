//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu async/await
#![allow(unused_imports)]
use {
    crate::{
        core::{
            compile::Compiler as _,
            exception::{self, Condition, Exception},
            frame::Frame,
            mu::Mu,
            types::{Tag, Type},
        },
        types::{
            cons::Cons,
            fixnum::Fixnum,
            function::Function,
            struct_::Struct,
            symbol::{Core as _, Symbol, UNBOUND},
        },
    },
    futures::executor::block_on,
    std::{assert, sync},
};

pub trait Compiler {
    fn compile_alambda(_: &Mu, _: Tag) -> exception::Result<Tag>;
}

impl Compiler for Mu {
    fn compile_alambda(mu: &Mu, args: Tag) -> exception::Result<Tag> {
        let (lambda, body) = match Tag::type_of(mu, args) {
            Type::Cons => {
                let lambda = Cons::car(mu, args);

                match Tag::type_of(mu, lambda) {
                    Type::Null | Type::Cons => (lambda, Cons::cdr(mu, args)),
                    _ => return Err(Exception::new(Condition::Type, "lambda", args)),
                }
            }
            _ => return Err(Exception::new(Condition::Syntax, "alambda", args)),
        };

        let id = Symbol::new(mu, Tag::nil(), "lambda", Tag::nil()).evict(mu);

        match Self::compile_frame_symbols(mu, lambda) {
            Ok(lexicals) => {
                let mut lexenv_ref = mu.compile.write().unwrap();
                lexenv_ref.push((id, lexicals));
            }
            Err(e) => return Err(e),
        };

        // add async decoration here
        let form = match Self::compile_list(mu, body) {
            Ok(form) => match Cons::length(mu, lambda) {
                Some(len) => Ok(Function::new(Fixnum::as_tag(len as i64), form, id).evict(mu)),
                None => panic!(":alambda"),
            },
            Err(e) => Err(e),
        };

        let mut lexenv_ref = mu.compile.write().unwrap();
        lexenv_ref.pop();

        form
    }
}

pub trait MuFunction {
    fn mu_async(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_await(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_async(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let func = fp.argv[0];
        let args = fp.argv[1];

        async fn async_apply() {
            Tag::nil();
        }

        fp.value = match Tag::type_of(mu, func) {
            Type::Function => match Tag::type_of(mu, args) {
                Type::Cons | Type::Null => {
                    let future = async_apply();
                    block_on(future);
                    Tag::nil()
                }
                _ => return Err(Exception::new(Condition::Type, "async", args)),
            },
            _ => return Err(Exception::new(Condition::Type, "async", func)),
        };

        Ok(())
    }

    fn mu_await(_mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = Tag::nil();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn event() {
        assert_eq!(2 + 2, 4);
    }
}
