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
            namespace::Scope,
            struct_::Struct,
            symbol::{Core as _, Symbol, UNBOUND},
        },
    },
    std::{assert, sync, thread},
};

pub struct Async {
    pub handler: thread::JoinHandle<std::result::Result<Tag, Exception>>,
    pub start: sync::Mutex<u64>,
}

pub trait MuFunction {
    fn mu_async(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_await(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Async {
    fn mu_async(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let func = fp.argv[0];
        let args = fp.argv[1];

        fp.value = match Tag::type_of(mu, func) {
            Type::Function => match Tag::type_of(mu, args) {
                Type::Cons | Type::Null => {
                    let sym =
                        Symbol::new(mu, Tag::nil(), Scope::Extern, "thread", *UNBOUND).evict(mu);
                    // let start = sync::Mutex::new(0);
                    let builder = thread::Builder::new();
                    // start.lock();

                    let _handler: thread::JoinHandle<exception::Result<Tag>> = builder
                        .spawn(|| {
                            // sync::Mutex::lock(&start);
                            let _value = Tag::nil();
                            // let _argv = args;
                            // Frame { func, argv, value }.apply(mu, func)
                            Ok(Tag::nil())
                        })
                        .unwrap();

                    // let mut event_map_ref = mu.event_map.write().unwrap();
                    // assert!(event_map_ref.contains_key(&sym.as_u64()));
                    // event_map_ref.insert( sym.as_u64(), Event { handler, start } );

                    Struct::to_tag(mu, Symbol::keyword(":event"), vec![sym, func, args])
                }
                _ => return Err(Exception::new(Condition::Type, "make-ev", args)),
            },
            _ => return Err(Exception::new(Condition::Type, "make-ev", func)),
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
