//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu namespace symbols
use {
    crate::{
        core::{
            exception::{self, Condition, Exception},
            frame::Frame,
            indirect::IndirectTag,
            mu::{Core as _, Mu},
            types::{Tag, TagType, Type},
        },
        types::{
            symbol::{Core as _, Symbol, UNBOUND},
            vecimage::{TypedVec, VecType},
            vector::{Core as _, Vector},
        },
    },
    std::{collections::HashMap, str, sync::RwLock},
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
        let mut ns_ref = mu.ns_map.write().unwrap();

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
        let ns_ref = mu.ns_map.read().unwrap();

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
        let ns_ref = mu.ns_map.read().unwrap();
        let (_, ns_cache) = &ns_ref[&ns.as_u64()];
        let hash = ns_cache.read().unwrap();

        if hash.contains_key(name) {
            Some(hash[name])
        } else {
            None
        }
    }

    fn intern(mu: &Mu, ns: Tag, symbol: Tag) {
        let ns_ref = mu.ns_map.read().unwrap();
        let (_, ns_cache) = &ns_ref[&ns.as_u64()];
        let name = Vector::as_string(mu, Symbol::name(mu, symbol));

        let mut hash = ns_cache.write().unwrap();
        hash.insert(name, symbol);
    }
}

pub struct Namespace {
    name: Tag,   // string
    import: Tag, // import namespace
}

impl Namespace {
    pub fn new(mu: &Mu, name: &str, import: Tag) -> Self {
        match Tag::type_of(mu, import) {
            Type::Null | Type::Namespace => Namespace {
                name: Vector::from_string(name).evict(mu),
                import,
            },
            _ => panic!(),
        }
    }

    pub fn evict(&self, mu: &Mu) -> Tag {
        let image: &[[u8; 8]] = &[self.name.as_slice(), self.import.as_slice()];

        let mut heap_ref = mu.heap.write().unwrap();
        Tag::Indirect(
            IndirectTag::new()
                .with_offset(heap_ref.alloc(image, Type::Namespace as u8) as u64)
                .with_heap_id(1)
                .with_tag(TagType::Indirect),
        )
    }

    pub fn to_image(mu: &Mu, tag: Tag) -> Self {
        match Tag::type_of(mu, tag) {
            Type::Namespace => match tag {
                Tag::Indirect(main) => {
                    let heap_ref = mu.heap.write().unwrap();
                    Namespace {
                        name: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize, 8).unwrap(),
                        ),
                        import: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize + 8, 8).unwrap(),
                        ),
                    }
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn name(mu: &Mu, ns: Tag) -> Tag {
        match Tag::type_of(mu, ns) {
            Type::Namespace => match ns {
                Tag::Indirect(_) => Self::to_image(mu, ns).name,
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn import(mu: &Mu, ns: Tag) -> Tag {
        match Tag::type_of(mu, ns) {
            Type::Namespace => match ns {
                Tag::Indirect(_) => Self::to_image(mu, ns).import,
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
}

pub trait Core {
    fn intern(_: &Mu, _: Tag, _: String, _: Tag) -> Tag;
    fn view(_: &Mu, _: Tag) -> Tag;
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
}

impl Core for Namespace {
    fn view(mu: &Mu, ns: Tag) -> Tag {
        let vec = vec![Self::name(mu, ns), Self::import(mu, ns)];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn write(mu: &Mu, ns: Tag, _: bool, stream: Tag) -> exception::Result<()> {
        match Tag::type_of(mu, ns) {
            Type::Namespace => {
                let name = Self::name(mu, ns);
                match mu.write_string("#<namespace: ".to_string(), stream) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
                match mu.write(name, true, stream) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
                mu.write_string(">".to_string(), stream)
            }
            _ => panic!(),
        }
    }

    fn intern(mu: &Mu, ns: Tag, name: String, value: Tag) -> Tag {
        match Tag::type_of(mu, ns) {
            Type::Namespace => match ns {
                Tag::Indirect(_) => match <Mu as NSMaps>::map(mu, ns, &name) {
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

                            let mut heap_ref = mu.heap.write().unwrap();
                            heap_ref.write_image(slices, offset);
                        }

                        symbol
                    }
                    None => {
                        let symbol = Symbol::new(
                            mu,
                            if ns.eq_(mu.unintern_ns) {
                                Tag::nil()
                            } else {
                                ns
                            },
                            &name,
                            value,
                        )
                        .evict(mu);

                        <Mu as NSMaps>::intern(mu, ns, symbol);

                        let image = Self::to_image(mu, ns);

                        let slices: &[[u8; 8]] = &[image.name.as_slice(), image.import.as_slice()];

                        let offset = match ns {
                            Tag::Indirect(heap) => heap.offset(),
                            _ => panic!(),
                        } as usize;

                        let mut heap_ref = mu.heap.write().unwrap();
                        heap_ref.write_image(slices, offset);

                        symbol
                    }
                },
                _ => panic!(),
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
    fn mu_ns_import(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_ns_name(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_ns_symbols(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Namespace {
    fn mu_untern(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];
        let name = fp.argv[1];

        let ns = match Tag::type_of(mu, ns) {
            Type::Null => mu.unintern_ns,
            Type::Namespace => ns,
            _ => return Err(Exception::new(Condition::Type, "untern", ns)),
        };

        fp.value = match Tag::type_of(mu, ns) {
            Type::Namespace => match Tag::type_of(mu, name) {
                Type::Vector if Vector::type_of(mu, name) == Type::Char => {
                    Self::intern(mu, ns, Vector::as_string(mu, name), *UNBOUND)
                }
                _ => return Err(Exception::new(Condition::Type, "untern", name)),
            },
            _ => return Err(Exception::new(Condition::Type, "untern", ns)),
        };

        Ok(())
    }

    fn mu_intern(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];
        let name = fp.argv[1];
        let value = fp.argv[2];

        let ns = match Tag::type_of(mu, ns) {
            Type::Null => mu.unintern_ns,
            Type::Namespace => ns,
            _ => return Err(Exception::new(Condition::Type, "intern", ns)),
        };

        fp.value = match Tag::type_of(mu, ns) {
            Type::Namespace => match Tag::type_of(mu, name) {
                Type::Vector if Vector::type_of(mu, name) == Type::Char => {
                    Self::intern(mu, ns, Vector::as_string(mu, name), value)
                }
                _ => return Err(Exception::new(Condition::Type, "intern", name)),
            },
            _ => return Err(Exception::new(Condition::Type, "intern", ns)),
        };

        Ok(())
    }

    fn mu_make_ns(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let name = fp.argv[0];
        let import = fp.argv[1];

        match Tag::type_of(mu, name) {
            Type::Vector => match Tag::type_of(mu, import) {
                Type::Null | Type::Namespace => {
                    fp.value = Self::new(mu, &Vector::as_string(mu, name), import).evict(mu);
                    <Mu as NSMaps>::add_ns(mu, fp.value).unwrap();
                    Ok(())
                }
                _ => Err(Exception::new(Condition::Type, "make-ns", import)),
            },
            _ => Err(Exception::new(Condition::Type, "make-ns", name)),
        }
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
            Type::Null => mu.unintern_ns,
            Type::Namespace => ns,
            _ => return Err(Exception::new(Condition::Type, "intern", ns)),
        };

        match Tag::type_of(mu, name) {
            Type::Vector => match Tag::type_of(mu, ns) {
                Type::Namespace => {
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

    fn mu_ns_import(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];

        match Tag::type_of(mu, ns) {
            Type::Namespace => {
                fp.value = Self::import(mu, ns);
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "ns-imp", ns)),
        }
    }

    fn mu_ns_name(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];

        match Tag::type_of(mu, ns) {
            Type::Namespace => {
                fp.value = Self::name(mu, ns);
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "ns-name", ns)),
        }
    }

    fn mu_ns_symbols(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];

        match Tag::type_of(mu, ns) {
            Type::Namespace => {
                fp.value = Tag::nil();
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "ns-syms", ns)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn namespace() {
        assert_eq!(true, true)
    }
}
