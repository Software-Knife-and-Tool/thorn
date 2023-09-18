//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu async/await
#![allow(unused_imports)]
use {
    crate::{
        core::{
            compile::Compiler as _,
            direct::{DirectInfo, DirectTag, DirectType, ExtType},
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
    futures::{future::BoxFuture, FutureExt},
    futures_locks::RwLock,
    std::assert,
};

pub trait MuFunction {
    fn mu_async(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_await(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_abort(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_async(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let func = fp.argv[0];
        let args = fp.argv[1];

        fp.value = match Tag::type_of(mu, func) {
            Type::Function => match Tag::type_of(mu, args) {
                Type::Cons | Type::Null => {
                    let mut _map_ref = mu.async_map.write();
                    let tag = Tag::to_direct(
                        0 as u64,
                        DirectInfo::ExtType(ExtType::Async),
                        DirectType::Ext,
                    );

                    let _future: BoxFuture<'static, Tag> = Box::pin(async { Tag::nil() });

                    /*
                    map_ref.insert(
                        tag.as_u64(),
                        future,
                    );
                    */

                    tag
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

    fn mu_abort(_mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
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
