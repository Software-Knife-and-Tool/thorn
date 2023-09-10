//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu environment
//!    Mu
use {
    crate::{
        core::{
            exception,
            exception::{Condition, Exception},
            frame::Frame,
            functions::{Core as _, LibMuFunction},
            namespace::{Core as _, NSMaps, Namespace},
            reader::{Core as _, Reader},
            types::{Tag, Type},
        },
        image::heap::Heap,
        system::sys as system,
        types::{
            char::{Char, Core as _},
            cons::{Cons, ConsIter, Core as _},
            fixnum::{Core as _, Fixnum},
            float::{Core as _, Float},
            function::{Core as _, Function},
            stream::{Core as _, Stream},
            streambuilder::StreamBuilder,
            struct_::{Core as _, Struct},
            symbol::{Core as _, Symbol},
            vector::{Core as _, Vector},
        },
    },
    cpu_time::ProcessTime,
    std::{collections::HashMap, sync::RwLock},
};

// mu environment
pub struct Mu {
    pub config: String,
    pub version: Tag,

    // heap
    pub heap: RwLock<Heap>,

    // environments
    pub compile: RwLock<Vec<(Tag, Vec<Tag>)>>,
    pub dynamic: RwLock<Vec<(u64, usize)>>,
    pub lexical: RwLock<HashMap<u64, RwLock<Vec<Frame>>>>,

    // exception dynamic unwind stack
    pub unwind: RwLock<Vec<usize>>,

    // functions
    pub functions: Vec<LibMuFunction>,

    // namespaces
    pub keyword_ns: Tag,
    pub mu_ns: Tag,
    pub null_ns: Tag,

    // reader
    pub reader: Reader,

    // standard streams
    pub stdin: Tag,
    pub stdout: Tag,
    pub errout: Tag,

    // namespace map/symbol caches
    pub ns_map: RwLock<<Mu as NSMaps>::NSMap>,

    // system
    pub start_time: ProcessTime,
    pub system: system::System,

    // async map
    pub async_map: RwLock<HashMap<u64, Tag>>,
}

pub trait Core {
    const VERSION: &'static str = "0.0.12";

    fn new(config: String) -> Self;
    fn apply(&self, _: Tag, _: Tag) -> exception::Result<Tag>;
    fn apply_(&self, _: Tag, _: Vec<Tag>) -> exception::Result<Tag>;
    fn eval(&self, _: Tag) -> exception::Result<Tag>;
    fn write(&self, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn write_string(&self, _: String, _: Tag) -> exception::Result<()>;
}

impl Core for Mu {
    fn new(config: String) -> Self {
        let mut mu = Mu {
            config,
            version: Tag::nil(),

            // heap
            heap: RwLock::new(Heap::new(1024)),

            // environments
            compile: RwLock::new(Vec::new()),
            dynamic: RwLock::new(Vec::new()),
            lexical: RwLock::new(HashMap::new()),

            // exception unwind stack
            unwind: RwLock::new(Vec::new()),

            // functions
            functions: Vec::new(),

            // namespaces
            keyword_ns: Tag::nil(),
            mu_ns: Tag::nil(),
            null_ns: Tag::nil(),
            ns_map: RwLock::new(HashMap::new()),

            // streams
            stdin: Tag::nil(),
            stdout: Tag::nil(),
            errout: Tag::nil(),

            // reader
            reader: Reader::new(),

            // system
            start_time: ProcessTime::now(),
            system: system::System::new(),

            // async
            async_map: RwLock::new(HashMap::new()),
        };

        // establish the namespaces first
        mu.mu_ns = Namespace::new("mu").evict(&mu);
        match <Mu as NSMaps>::add_ns(&mu, mu.mu_ns) {
            Ok(_) => (),
            Err(_) => panic!(),
        };

        mu.null_ns = Namespace::new("").evict(&mu);
        match <Mu as NSMaps>::add_ns(&mu, mu.null_ns) {
            Ok(_) => (),
            Err(_) => panic!(),
        };

        mu.keyword_ns = Namespace::new("keyword").evict(&mu);
        match <Mu as NSMaps>::add_ns(&mu, mu.keyword_ns) {
            Ok(_) => (),
            Err(_) => panic!(),
        };

        // the version string
        mu.version = Vector::from_string(<Mu as Core>::VERSION).evict(&mu);
        Namespace::intern(&mu, mu.mu_ns, "version".to_string(), mu.version);

        // the standard streams
        mu.stdin = match StreamBuilder::new().stdin().build(&mu) {
            Ok(stdin) => stdin.evict(&mu),
            Err(_) => panic!(),
        };
        Namespace::intern(&mu, mu.mu_ns, "std-in".to_string(), mu.stdin);

        mu.stdout = match StreamBuilder::new().stdout().build(&mu) {
            Ok(stdout) => stdout.evict(&mu),
            Err(_) => panic!(),
        };
        Namespace::intern(&mu, mu.mu_ns, "std-out".to_string(), mu.stdout);

        mu.errout = match StreamBuilder::new().errout().build(&mu) {
            Ok(errout) => errout.evict(&mu),
            Err(_) => panic!(),
        };
        Namespace::intern(&mu, mu.mu_ns, "err-out".to_string(), mu.errout);

        // mu functions
        mu.functions = Self::install_lib_functions(&mu);

        // the reader, has to be last
        mu.reader = mu.reader.build(&mu);

        mu
    }

    fn apply_(&self, func: Tag, argv: Vec<Tag>) -> exception::Result<Tag> {
        let value = Tag::nil();

        Frame { func, argv, value }.apply(self, func)
    }

    fn apply(&self, func: Tag, args: Tag) -> exception::Result<Tag> {
        let value = Tag::nil();
        let mut argv = Vec::new();

        for cons in ConsIter::new(self, args) {
            match self.eval(Cons::car(self, cons)) {
                Ok(arg) => argv.push(arg),
                Err(e) => return Err(e),
            }
        }

        Frame { func, argv, value }.apply(self, func)
    }

    fn eval(&self, expr: Tag) -> exception::Result<Tag> {
        match Tag::type_of(self, expr) {
            Type::Cons => {
                let func = Cons::car(self, expr);
                let args = Cons::cdr(self, expr);
                match Tag::type_of(self, func) {
                    Type::Keyword if func.eq_(Symbol::keyword("quote")) => {
                        Ok(Cons::car(self, args))
                    }
                    Type::Symbol => {
                        if Symbol::is_unbound(self, func) {
                            Err(Exception::new(Condition::Unbound, "eval", func))
                        } else {
                            let fnc = Symbol::value(self, func);
                            match Tag::type_of(self, fnc) {
                                Type::Function => self.apply(fnc, args),
                                _ => Err(Exception::new(Condition::Type, "eval", func)),
                            }
                        }
                    }
                    Type::Function => self.apply(func, args),
                    _ => Err(Exception::new(Condition::Type, "eval", func)),
                }
            }
            Type::Symbol => {
                if Symbol::is_unbound(self, expr) {
                    Err(Exception::new(Condition::Unbound, "eval", expr))
                } else {
                    Ok(Symbol::value(self, expr))
                }
            }
            _ => Ok(expr),
        }
    }

    fn write(&self, tag: Tag, escape: bool, stream: Tag) -> exception::Result<()> {
        if Tag::type_of(self, stream) != Type::Stream {
            panic!("{:?}", Tag::type_of(self, stream))
        }

        match Tag::type_of(self, tag) {
            Type::Char => Char::write(self, tag, escape, stream),
            Type::Cons => Cons::write(self, tag, escape, stream),
            Type::Fixnum => Fixnum::write(self, tag, escape, stream),
            Type::Float => Float::write(self, tag, escape, stream),
            Type::Function => Function::write(self, tag, escape, stream),
            Type::Null | Type::Symbol | Type::Keyword => Symbol::write(self, tag, escape, stream),
            Type::Stream => Stream::write(self, tag, escape, stream),
            Type::Vector => Vector::write(self, tag, escape, stream),
            Type::Struct => Struct::write(self, tag, escape, stream),
            _ => panic!(),
        }
    }

    fn write_string(&self, str: String, stream: Tag) -> exception::Result<()> {
        if Tag::type_of(self, stream) != Type::Stream {
            panic!("{:?}", Tag::type_of(self, stream))
        }
        for ch in str.chars() {
            match Stream::write_char(self, stream, ch) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mu() {
        assert_eq!(2 + 2, 4);
    }
}
