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
            mu::{Core as _, Mu},
            stream::Core as _,
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
    futures::{executor::block_on, future::BoxFuture, FutureExt},
    futures_locks::RwLock,
    std::assert,
};

pub struct AsyncContext {
    pub func: Tag,
    pub args: Tag,
    pub context: <AsyncContext as Core>::AsyncFuture,
}

pub trait Core {
    type AsyncFuture;

    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn async_context(_: &Mu, _: Tag, _: Tag) -> exception::Result<Tag>;
}

impl Core for AsyncContext {
    type AsyncFuture = BoxFuture<'static, Result<Tag, Exception>>;

    fn async_context(mu: &Mu, func: Tag, args: Tag) -> exception::Result<Tag> {
        let async_id = match Tag::type_of(func) {
            Type::Function => match Tag::type_of(args) {
                Type::Cons | Type::Null => {
                    let mut map_ref = block_on(mu.async_map.write());
                    let mut async_id: u64 = map_ref.len() as u64;

                    let mut tag = DirectTag::to_direct(
                        async_id,
                        DirectInfo::ExtType(ExtType::AsyncId),
                        DirectType::Ext,
                    );

                    let future: <AsyncContext as Core>::AsyncFuture = Box::pin(async {
                        // mu.apply(func, args)
                        Ok(Tag::nil())
                    });

                    loop {
                        match map_ref.get(&tag.as_u64()) {
                            Some(_) => {
                                async_id += 1;
                                tag = DirectTag::to_direct(
                                    async_id,
                                    DirectInfo::ExtType(ExtType::AsyncId),
                                    DirectType::Ext,
                                );
                                continue;
                            }
                            None => {
                                map_ref.insert(
                                    tag.as_u64(),
                                    AsyncContext {
                                        func,
                                        args,
                                        context: future,
                                    },
                                );
                                break;
                            }
                        }
                    }

                    tag
                }
                _ => return Err(Exception::new(Condition::Type, "async", args)),
            },
            _ => return Err(Exception::new(Condition::Type, "async", func)),
        };

        Ok(async_id)
    }

    fn write(mu: &Mu, tag: Tag, _: bool, stream: Tag) -> exception::Result<()> {
        mu.write_string(format!("#<:asyncid [id:{}]>", Tag::data(&tag, mu)), stream)
    }
}

pub trait MuFunction {
    fn mu_await(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_abort(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_await(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let async_id = fp.argv[0];

        fp.value = match Tag::type_of(async_id) {
            Type::AsyncId => {
                let map_ref = block_on(mu.async_map.write());

                match map_ref.get(&async_id.as_u64()) {
                    Some(_future) => Tag::nil(), // async {
                    _ => return Err(Exception::new(Condition::Range, "await", async_id)),
                }
            }
            _ => return Err(Exception::new(Condition::Type, "await", async_id)),
        };

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
