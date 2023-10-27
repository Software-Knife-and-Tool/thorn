//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu environment
//!    Mu
use {
    crate::{
        core::{
            async_context::AsyncContext,
            config::Config,
            exception::{self, Condition, Exception},
            frame::Frame,
            funcall::{Core as _, LibMuFunction},
            namespace::{Cache, Core as NSCore},
            reader::{Core as _, Reader},
            types::{Tag, Type},
        },
        heap::bump_heap::BumpHeap,
        system::sys as system,
        types::{
            cons::{Cons, ConsIter, Core as _},
            function::{Core as _, Function},
            map::{Core as _, Map},
            r#struct::{Core as _, Struct},
            stream::{Core as _, Stream},
            streambuilder::StreamBuilder,
            symbol::{Core as _, Symbol},
            vector::{Core as _, Vector},
        },
    },
    cpu_time::ProcessTime,
    std::collections::HashMap,
};

// locking protocols
use {futures::executor::block_on, futures_locks::RwLock};

// mu environment
pub struct Mu {
    config: Config,
    pub version: Tag,

    // heap
    pub heap: RwLock<BumpHeap>,
    pub gc_root: RwLock<Vec<Tag>>,
    pub free: RwLock<Vec<Vec<usize>>>,

    // async environments
    pub compile: RwLock<Vec<(Tag, Vec<Tag>)>>,
    pub dynamic: RwLock<Vec<(u64, usize)>>,
    pub lexical: RwLock<HashMap<u64, RwLock<Vec<Frame>>>>,

    // async context map
    pub async_map: RwLock<HashMap<u64, AsyncContext>>,

    // exception dynamic unwind stack
    pub unwind: RwLock<Vec<usize>>,

    // map cache index
    pub map_index: RwLock<HashMap<usize, HashMap<u64, Tag>>>,

    // namespace map/symbol caches
    pub ns_map: RwLock<<Mu as Cache>::NSIndex>,

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

    // system
    pub start_time: ProcessTime,
    pub system: system::System,
}

pub trait Core {
    const VERSION: &'static str = "0.0.23";

    fn new(config: &Config) -> Self;
    fn apply(&self, _: Tag, _: Tag) -> exception::Result<Tag>;
    fn apply_(&self, _: Tag, _: Vec<Tag>) -> exception::Result<Tag>;
    fn eval(&self, _: Tag) -> exception::Result<Tag>;
    fn gc(&self) -> exception::Result<bool>;
    fn gc_mark(&self, _: Tag);
}

impl Core for Mu {
    fn new(config: &Config) -> Self {
        let mut mu = Mu {
            config: *config,
            version: Tag::nil(),

            // heap
            heap: RwLock::new(BumpHeap::new(config.npages)),
            gc_root: RwLock::new(Vec::<Tag>::new()),
            free: RwLock::new(Vec::<Vec<usize>>::new()),

            // async contexts
            async_map: RwLock::new(HashMap::new()),

            // async environments
            compile: RwLock::new(Vec::new()),
            dynamic: RwLock::new(Vec::new()),
            lexical: RwLock::new(HashMap::new()),

            // exception unwind stack
            unwind: RwLock::new(Vec::new()),

            // map caches
            map_index: RwLock::new(HashMap::new()),

            // namespace maps
            ns_map: RwLock::new(HashMap::new()),

            // functions
            functions: Vec::new(),

            // namespaces
            keyword_ns: Tag::nil(),
            mu_ns: Tag::nil(),
            null_ns: Tag::nil(),

            // streams
            stdin: Tag::nil(),
            stdout: Tag::nil(),
            errout: Tag::nil(),

            // reader
            reader: Reader::new(),

            // system
            start_time: ProcessTime::now(),
            system: system::System::new(),
        };

        // establish the namespaces first
        mu.keyword_ns = Symbol::keyword("keyword");

        mu.mu_ns = Symbol::keyword("mu");
        match <Mu as Cache>::add_ns(&mu, mu.mu_ns) {
            Ok(_) => (),
            Err(_) => panic!(),
        };

        mu.null_ns = Tag::nil();
        match <Mu as Cache>::add_ns(&mu, mu.null_ns) {
            Ok(_) => (),
            Err(_) => panic!(),
        };

        // the version string
        mu.version = Vector::from_string(<Mu as Core>::VERSION).evict(&mu);
        <Mu as NSCore>::intern_symbol(&mu, mu.mu_ns, "version".to_string(), mu.version);

        // the standard streams
        mu.stdin = match StreamBuilder::new().stdin().build(&mu) {
            Ok(stdin) => stdin.evict(&mu),
            Err(_) => panic!(),
        };
        <Mu as NSCore>::intern_symbol(&mu, mu.mu_ns, "std-in".to_string(), mu.stdin);

        mu.stdout = match StreamBuilder::new().stdout().build(&mu) {
            Ok(stdout) => stdout.evict(&mu),
            Err(_) => panic!(),
        };
        <Mu as NSCore>::intern_symbol(&mu, mu.mu_ns, "std-out".to_string(), mu.stdout);

        mu.errout = match StreamBuilder::new().errout().build(&mu) {
            Ok(errout) => errout.evict(&mu),
            Err(_) => panic!(),
        };
        <Mu as NSCore>::intern_symbol(&mu, mu.mu_ns, "err-out".to_string(), mu.errout);

        // heap
        {
            let mut free_ref = block_on(mu.free.write());

            for _i in 0..16 {
                free_ref.push(Vec::<usize>::new())
            }
        }

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
        match Tag::type_of(expr) {
            Type::Cons => {
                let func = Cons::car(self, expr);
                let args = Cons::cdr(self, expr);
                match Tag::type_of(func) {
                    Type::Keyword if func.eq_(Symbol::keyword("quote")) => {
                        Ok(Cons::car(self, args))
                    }
                    Type::Symbol => {
                        if Symbol::is_unbound(self, func) {
                            Err(Exception::new(Condition::Unbound, "eval", func))
                        } else {
                            let fnc = Symbol::value(self, func);
                            match Tag::type_of(fnc) {
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

    fn gc_mark(&self, tag: Tag) {
        match tag {
            Tag::Direct(_) => (),
            Tag::Indirect(_) => match Tag::type_of(tag) {
                Type::Cons => Cons::gc_mark(self, tag),
                Type::Function => Function::gc_mark(self, tag),
                Type::Map => Map::gc_mark(self, tag),
                Type::Stream => Stream::gc_mark(self, tag),
                Type::Struct => Struct::gc_mark(self, tag),
                Type::Symbol => Symbol::gc_mark(self, tag),
                Type::Vector => Vector::gc_mark(self, tag),
                _ => (),
            },
        }
    }

    fn gc(&self) -> exception::Result<bool> {
        let mut heap_ref = block_on(self.heap.write());
        let root_ref = block_on(self.gc_root.write());

        heap_ref.clear_refbits();

        for tag in &*root_ref {
            self.gc_mark(*tag)
        }

        heap_ref.sweep();
        heap_ref.dump_stats();

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mu() {
        assert_eq!(2 + 2, 4);
    }
}
