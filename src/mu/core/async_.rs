//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu async/await
#![allow(unused_imports)]
use {
    crate::{
        core::{
            exception::{self, Condition, Exception},
            frame::Frame,
            mu::Mu,
            types::{Tag, Type},
        },
        types::{
            struct_::Struct,
            symbol::{Core as _, Symbol, UNBOUND},
        },
    },
    futures::executor::block_on,
    std::{assert, sync},
};

async fn async_apply() {
    Tag::nil();
}

pub trait MuFunction {
    fn mu_async(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_await(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_async(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let _tag = fp.argv[0];
        let func = fp.argv[1];
        let args = fp.argv[2];

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
