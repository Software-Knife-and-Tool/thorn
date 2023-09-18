//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu namespace symbols
use {
    crate::{
        core::{
            exception::{self, Condition, Exception},
            frame::Frame,
            mu::Mu,
            types::{Tag, Type},
        },
        types::{
            cons::{Cons, Core as _},
            struct_::{Core as _, Struct},
            symbol::{Core as _, Symbol, UNBOUND},
            vector::{Core as _, Vector},
        },
    },
    futures::executor::block_on,
    futures_locks::RwLock,
    std::{collections::HashMap, str},
};

pub trait NSMaps {
    type NSCache;
    type NSMap;

    fn add_ns(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn intern(_: &Mu, _: Tag, _: Tag);
    fn map(_: &Mu, _: Tag, _: &str) -> Option<Tag>;
    fn map_ns(_: &Mu, _: &str) -> Option<Tag>;
}

impl NSMaps for Mu {
    type NSCache = RwLock<HashMap<String, Tag>>;
    type NSMap = HashMap<u64, (Tag, Self::NSCache)>;

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

    fn map_ns(mu: &Mu, name: &str) -> Option<Tag> {
        let ns_ref = block_on(mu.ns_map.read());

        for (_, ns) in ns_ref.iter() {
            let (ns_name, _) = ns;
            let map_name = Vector::as_string(mu, Namespace::name(mu, *ns_name));

            if name == map_name {
                return Some(*ns_name);
            }
        }

        None
    }

    fn map(mu: &Mu, ns: Tag, name: &str) -> Option<Tag> {
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
}

pub struct Namespace {
    name: String,
}

impl Namespace {
    pub fn new(name: &str) -> Self {
        Namespace {
            name: name.to_string(),
        }
    }

    pub fn is_ns(mu: &Mu, tag: Tag) -> Option<Tag> {
        match Tag::type_of(mu, tag) {
            Type::Struct => {
                let type_ = Struct::stype(mu, tag);
                if type_.eq_(Symbol::keyword("ns")) {
                    Some(tag)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn evict(&self, mu: &Mu) -> Tag {
        Struct::new(
            mu,
            "ns".to_string(),
            vec![Vector::from_string(&self.name).evict(mu)],
        )
        .evict(mu)
    }

    pub fn to_image(mu: &Mu, ns: Tag) -> Self {
        match Self::is_ns(mu, ns) {
            Some(ns) => {
                let struct_ = Struct::to_image(mu, ns);
                Namespace {
                    name: Vector::as_string(mu, Vector::ref_(mu, struct_.vector, 0).unwrap()),
                }
            }
            _ => panic!(),
        }
    }

    pub fn name(mu: &Mu, ns: Tag) -> Tag {
        match Self::is_ns(mu, ns) {
            Some(ns) => {
                let struct_ = Struct::to_image(mu, ns);
                Vector::ref_(mu, struct_.vector, 0).unwrap()
            }
            _ => panic!(),
        }
    }
}

pub trait Core {
    fn intern(_: &Mu, _: Tag, _: String, _: Tag) -> Tag;
}

impl Core for Namespace {
    fn intern(mu: &Mu, ns: Tag, name: String, value: Tag) -> Tag {
        match Self::is_ns(mu, ns) {
            Some(ns) => match <Mu as NSMaps>::map(mu, ns, &name) {
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

                    <Mu as NSMaps>::intern(mu, ns, symbol);

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
    fn mu_map_ns(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_ns_find(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_ns_name(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_ns_symbols(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Namespace {
    fn mu_untern(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];
        let name = fp.argv[1];

        let ns = match Tag::type_of(mu, ns) {
            Type::Null => mu.null_ns,
            Type::Struct => match Self::is_ns(mu, ns) {
                Some(ns) => ns,
                _ => return Err(Exception::new(Condition::Type, "untern", ns)),
            },
            _ => return Err(Exception::new(Condition::Type, "untern", ns)),
        };

        fp.value = match Tag::type_of(mu, name) {
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
                    if len > Tag::DIRECT_STR_MAX {
                        return Err(Exception::new(Condition::Syntax, "untern", name));
                    }

                    Symbol::keyword(&name_str)
                } else {
                    Self::intern(mu, ns, name_str, *UNBOUND)
                }
            }
            _ => return Err(Exception::new(Condition::Type, "untern", ns)),
        };

        Ok(())
    }

    fn mu_intern(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];
        let name = fp.argv[1];
        let value = fp.argv[2];

        let ns = match Tag::type_of(mu, ns) {
            Type::Null => mu.null_ns,
            Type::Struct => match Self::is_ns(mu, ns) {
                Some(ns) => ns,
                _ => return Err(Exception::new(Condition::Type, "intern", ns)),
            },
            _ => return Err(Exception::new(Condition::Type, "intern", ns)),
        };

        fp.value = match Self::is_ns(mu, ns) {
            Some(ns) => match Tag::type_of(mu, name) {
                Type::Vector if Vector::type_of(mu, name) == Type::Char => {
                    if Vector::length(mu, name) == 0 {
                        return Err(Exception::new(Condition::Syntax, "intern", ns));
                    }

                    let name_str = Vector::as_string(mu, name);
                    let str = name_str.as_bytes();
                    let len = str.len();

                    if len == 0 {
                        return Err(Exception::new(Condition::Syntax, "intern", name));
                    }

                    if ns.eq_(mu.keyword_ns) {
                        if len > Tag::DIRECT_STR_MAX {
                            return Err(Exception::new(Condition::Syntax, "intern", name));
                        }

                        Symbol::keyword(&name_str)
                    } else {
                        Self::intern(mu, ns, name_str, value)
                    }
                }
                _ => return Err(Exception::new(Condition::Type, "intern", name)),
            },
            _ => return Err(Exception::new(Condition::Type, "intern", ns)),
        };

        Ok(())
    }

    fn mu_make_ns(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns_name = fp.argv[0];

        match Tag::type_of(mu, ns_name) {
            Type::Vector if Vector::type_of(mu, ns_name) == Type::Char => {
                fp.value = Self::new(&Vector::as_string(mu, ns_name)).evict(mu);
                <Mu as NSMaps>::add_ns(mu, fp.value).unwrap();
            }
            _ => return Err(Exception::new(Condition::Type, "make-ns", ns_name)),
        }

        Ok(())
    }

    fn mu_map_ns(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns_name = fp.argv[0];

        fp.value = match Tag::type_of(mu, ns_name) {
            Type::Vector if Vector::type_of(mu, ns_name) == Type::Char => {
                match <Mu as NSMaps>::map_ns(mu, &Vector::as_string(mu, ns_name)) {
                    Some(ns) => ns,
                    None => Tag::nil(),
                }
            }
            _ => return Err(Exception::new(Condition::Type, "map-ns", ns_name)),
        };

        Ok(())
    }

    fn mu_ns_find(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];
        let name = fp.argv[1];

        let ns = match Tag::type_of(mu, ns) {
            Type::Null => mu.null_ns,
            Type::Struct => match Self::is_ns(mu, ns) {
                Some(_) => ns,
                _ => return Err(Exception::new(Condition::Type, "intern", ns)),
            },
            _ => return Err(Exception::new(Condition::Type, "intern", ns)),
        };

        match Tag::type_of(mu, name) {
            Type::Vector => match Self::is_ns(mu, ns) {
                Some(_) => {
                    let ns_name = Vector::as_string(mu, Namespace::name(mu, ns));
                    let sy_name = Vector::as_string(mu, name);

                    fp.value = match <Mu as NSMaps>::map_ns(mu, &ns_name) {
                        Some(ns) => match <Mu as NSMaps>::map(mu, ns, &sy_name) {
                            Some(sym) => sym,
                            None => Tag::nil(),
                        },
                        _ => return Err(Exception::new(Condition::Range, "ns-find", name)),
                    };

                    Ok(())
                }
                _ => Err(Exception::new(Condition::Type, "ns-find", ns)),
            },
            _ => Err(Exception::new(Condition::Type, "ns-find", name)),
        }
    }

    fn mu_ns_name(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];

        match Self::is_ns(mu, ns) {
            Some(_) => {
                fp.value = Self::name(mu, ns);
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "ns-name", ns)),
        }
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
