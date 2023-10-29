//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu symbol namespaces
use {
    crate::{
        core::{
            direct::DirectTag,
            exception::{self, Condition, Exception},
            frame::Frame,
            mu::{Core as _, Mu},
            types::{Tag, Type},
        },
        types::{
            cons::{Cons, Core as _},
            symbol::{Core as _, Symbol, UNBOUND},
            vector::{Core as _, Vector},
        },
    },
    std::{collections::HashMap, str},
};

use {futures::executor::block_on, futures_locks::RwLock};

pub trait Cache {
    type NSCache;
    type NSIndex;

    fn add_ns(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn intern(_: &Mu, _: Tag, _: Tag);
    fn gc_ns(_: &Mu);
    fn is_ns(_: &Mu, _: Tag) -> Option<Tag>;
    fn map_symbol(_: &Mu, _: Tag, _: &str) -> Option<Tag>;
}

impl Cache for Mu {
    type NSCache = RwLock<HashMap<String, Tag>>;
    type NSIndex = HashMap<u64, (Tag, Self::NSCache)>;

    fn gc_ns(mu: &Mu) {
        let ns_map_ref = block_on(mu.ns_map.read());

        for cache in (*ns_map_ref).values() {
            let cache_ref = block_on(cache.1.read());
            for symbol in (*cache_ref).values() {
                Mu::gc_mark(mu, *symbol)
            }
        }
    }

    fn add_ns(mu: &Mu, ns: Tag) -> exception::Result<Tag> {
        let mut ns_ref = block_on(mu.ns_map.write());

        if ns_ref.contains_key(&ns.as_u64()) {
            return Err(Exception::new(Condition::Type, "make-ns", ns));
        }

        ns_ref.insert(
            ns.as_u64(),
            (ns, RwLock::new(HashMap::<String, Tag>::new())),
        );

        Ok(ns)
    }

    fn map_symbol(mu: &Mu, ns: Tag, name: &str) -> Option<Tag> {
        let ns_ref = block_on(mu.ns_map.read());

        let (_, ns_cache) = &ns_ref[&ns.as_u64()];

        let hash = block_on(ns_cache.read());

        if hash.contains_key(name) {
            Some(hash[name])
        } else {
            None
        }
    }

    fn intern(mu: &Mu, ns: Tag, symbol: Tag) {
        let ns_ref = block_on(mu.ns_map.read());

        let (_, ns_cache) = &ns_ref[&ns.as_u64()];
        let name = Vector::as_string(mu, Symbol::name(mu, symbol));

        let mut hash = block_on(ns_cache.write());

        hash.insert(name, symbol);
    }

    fn is_ns(mu: &Mu, tag: Tag) -> Option<Tag> {
        match Tag::type_of(tag) {
            Type::Null => Some(tag),
            Type::Keyword => {
                let ns_ref = block_on(mu.ns_map.read());

                if ns_ref.contains_key(&tag.as_u64()) {
                    Some(tag)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub trait Core {
    fn intern_symbol(_: &Mu, _: Tag, _: String, _: Tag) -> Tag;
}

impl Core for Mu {
    fn intern_symbol(mu: &Mu, ns: Tag, name: String, value: Tag) -> Tag {
        match Self::is_ns(mu, ns) {
            Some(ns) => match <Mu as Cache>::map_symbol(mu, ns, &name) {
                Some(symbol) => {
                    // if the symbol is unbound, bind it.
                    // otherwise, we ignore the new binding.
                    // this allows a reader to intern a functional
                    // symbol without binding it.
                    if Symbol::is_unbound(mu, symbol) {
                        let image = Symbol::to_image(mu, symbol);

                        let slices: &[[u8; 8]] = &[
                            image.namespace.as_slice(),
                            image.name.as_slice(),
                            value.as_slice(),
                        ];

                        let offset = match symbol {
                            Tag::Indirect(heap) => heap.offset(),
                            _ => panic!(),
                        } as usize;

                        let mut heap_ref = block_on(mu.heap.write());

                        heap_ref.write_image(slices, offset);
                    }

                    symbol
                }
                None => {
                    let symbol = Symbol::new(mu, ns, &name, value).evict(mu);

                    <Mu as Cache>::intern(mu, ns, symbol);

                    symbol
                }
            },
            _ => panic!(),
        }
    }
}

pub trait MuFunction {
    fn mu_intern(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_untern(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_make_ns(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_ns_find(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_ns_symbols(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_untern(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];
        let name = fp.argv[1];

        let ns = match Tag::type_of(ns) {
            Type::Null => mu.null_ns,
            Type::Keyword => match Self::is_ns(mu, ns) {
                Some(ns) => ns,
                _ => return Err(Exception::new(Condition::Type, "untern", ns)),
            },
            _ => return Err(Exception::new(Condition::Type, "untern", ns)),
        };

        fp.value = match Tag::type_of(name) {
            Type::Vector if Vector::type_of(mu, name) == Type::Char => {
                if Vector::length(mu, name) == 0 {
                    return Err(Exception::new(Condition::Syntax, "untern", ns));
                }

                let name_str = Vector::as_string(mu, name);
                let str = name_str.as_bytes();
                let len = str.len();

                if len == 0 {
                    return Err(Exception::new(Condition::Syntax, "untern", name));
                }

                if ns.eq_(mu.keyword_ns) {
                    if len > DirectTag::DIRECT_STR_MAX {
                        return Err(Exception::new(Condition::Syntax, "untern", name));
                    }

                    Symbol::keyword(&name_str)
                } else {
                    <Mu as Core>::intern_symbol(mu, ns, name_str, *UNBOUND)
                }
            }
            _ => return Err(Exception::new(Condition::Type, "untern", ns)),
        };

        Ok(())
    }

    fn mu_intern(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns_tag = fp.argv[0];
        let name = fp.argv[1];
        let value = fp.argv[2];

        let ns = match Tag::type_of(ns_tag) {
            Type::Null => mu.null_ns,
            Type::Keyword => match Self::is_ns(mu, ns_tag) {
                Some(ns) => ns,
                _ => return Err(Exception::new(Condition::Type, "intern", ns_tag)),
            },
            _ => return Err(Exception::new(Condition::Type, "intern", ns_tag)),
        };

        fp.value = match Self::is_ns(mu, ns) {
            Some(ns) => match Tag::type_of(name) {
                Type::Vector if Vector::type_of(mu, name) == Type::Char => {
                    if Vector::length(mu, name) == 0 {
                        return Err(Exception::new(Condition::Syntax, "intern", ns_tag));
                    }

                    let name_str = Vector::as_string(mu, name);
                    let str = name_str.as_bytes();
                    let len = str.len();

                    if len == 0 {
                        return Err(Exception::new(Condition::Syntax, "intern", name));
                    }

                    if ns.eq_(mu.keyword_ns) {
                        if len > DirectTag::DIRECT_STR_MAX {
                            return Err(Exception::new(Condition::Syntax, "intern", name));
                        }

                        Symbol::keyword(&name_str)
                    } else {
                        <Mu as Core>::intern_symbol(mu, ns, name_str, value)
                    }
                }
                _ => return Err(Exception::new(Condition::Type, "intern", name)),
            },
            _ => return Err(Exception::new(Condition::Type, "intern", ns)),
        };

        Ok(())
    }

    fn mu_make_ns(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns_tag = fp.argv[0];

        match Tag::type_of(ns_tag) {
            Type::Keyword => {
                fp.value = ns_tag;
                match Self::is_ns(mu, ns_tag) {
                    Some(_) => return Err(Exception::new(Condition::Namespace, "make-ns", ns_tag)),
                    None => <Mu as Cache>::add_ns(mu, fp.value).unwrap(),
                };
            }
            _ => return Err(Exception::new(Condition::Type, "make-ns", ns_tag)),
        }

        Ok(())
    }

    fn mu_ns_find(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns_tag = fp.argv[0];
        let name = fp.argv[1];

        let ns = match Tag::type_of(ns_tag) {
            Type::Null => mu.null_ns,
            Type::Keyword => match Self::is_ns(mu, ns_tag) {
                Some(_) => ns_tag,
                _ => return Err(Exception::new(Condition::Type, "intern", ns_tag)),
            },
            _ => return Err(Exception::new(Condition::Type, "intern", ns_tag)),
        };

        fp.value = match Tag::type_of(name) {
            Type::Vector if Vector::type_of(mu, name) == Type::Char => {
                match Self::map_symbol(mu, ns_tag, &Vector::as_string(mu, name)) {
                    Some(sym) => sym,
                    None => Tag::nil(),
                }
            }
            _ => return Err(Exception::new(Condition::Type, "ns-find", ns)),
        };

        Ok(())
    }

    fn mu_ns_symbols(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];

        fp.value = match Self::is_ns(mu, ns) {
            Some(_) => {
                let ns_ref = block_on(mu.ns_map.read());

                let (_, ns_cache) = &ns_ref[&ns.as_u64()];

                let hash = block_on(ns_cache.read());

                let mut vec = vec![];

                for key in hash.keys() {
                    vec.push(hash[key])
                }

                Cons::vlist(mu, &vec)
            }
            _ => return Err(Exception::new(Condition::Type, "ns-syms", ns)),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn namespace() {
        assert_eq!(true, true)
    }
}
