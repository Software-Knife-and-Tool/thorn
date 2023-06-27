//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu namespace type
use {
    crate::{
        core::{
            exception::{self, Condition, Exception},
            frame::Frame,
            indirect::IndirectTag,
            mu::{Core as _, Mu},
            nsmap::NSMaps,
            types::{Tag, TagType, Type},
        },
        types::{
            cons::{Cons, Core as _},
            symbol::{Core as _, Symbol, UNBOUND},
            vecimage::{TypedVec, VecType},
            vector::{Core as _, Vector},
        },
    },
    std::str,
};

#[derive(Copy, Clone, Debug)]
pub enum Scope {
    Intern,
    Extern,
}

pub struct Namespace {
    name: Tag,    // string
    externs: Tag, // list of symbols
    interns: Tag, // list of symbols
    import: Tag,  // import namespace
}

impl Namespace {
    pub fn new(mu: &Mu, name: &str, import: Tag) -> Self {
        match Tag::type_of(mu, import) {
            Type::Null | Type::Namespace => Namespace {
                name: Vector::from_string(name).evict(mu),
                externs: Tag::nil(),
                interns: Tag::nil(),
                import,
            },
            _ => panic!(),
        }
    }

    pub fn evict(&self, mu: &Mu) -> Tag {
        let image: &[[u8; 8]] = &[
            self.name.as_slice(),
            self.externs.as_slice(),
            self.interns.as_slice(),
            self.import.as_slice(),
        ];

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
                        externs: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize + 8, 8).unwrap(),
                        ),
                        interns: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize + 16, 8).unwrap(),
                        ),
                        import: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize + 24, 8).unwrap(),
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

    pub fn externs(mu: &Mu, ns: Tag) -> Tag {
        match Tag::type_of(mu, ns) {
            Type::Namespace => match ns {
                Tag::Indirect(_) => Self::to_image(mu, ns).externs,
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn interns(mu: &Mu, ns: Tag) -> Tag {
        match Tag::type_of(mu, ns) {
            Type::Namespace => match ns {
                Tag::Indirect(_) => Self::to_image(mu, ns).interns,
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
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn intern(_: &Mu, _: Tag, _: Scope, _: String, _: Tag) -> Tag;
    fn view(_: &Mu, _: Tag) -> Tag;
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

    fn intern(mu: &Mu, ns: Tag, scope: Scope, name: String, value: Tag) -> Tag {
        match Tag::type_of(mu, ns) {
            Type::Namespace => match ns {
                Tag::Indirect(_) => match <Mu as NSMaps>::map(mu, ns, scope, &name) {
                    Some(symbol) => {
                        // if the symbol is unbound, bind it.
                        // otherwise, we ignore the new binding.
                        // this allows a reader to intern a functional
                        // symbol without binding it.
                        if Symbol::is_unbound(mu, symbol) {
                            let image = Symbol::to_image(mu, symbol);

                            let slices: &[[u8; 8]] = &[
                                image.namespace.as_slice(),
                                image.scope.as_slice(),
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
                            scope,
                            &name,
                            value,
                        )
                        .evict(mu);

                        <Mu as NSMaps>::intern(mu, ns, scope, symbol);

                        let mut image = Self::to_image(mu, ns);
                        match scope {
                            Scope::Intern => {
                                image.interns = Cons::new(symbol, image.interns).evict(mu)
                            }
                            Scope::Extern => {
                                image.externs = Cons::new(symbol, image.externs).evict(mu)
                            }
                        };

                        let slices: &[[u8; 8]] = &[
                            image.name.as_slice(),
                            image.externs.as_slice(),
                            image.interns.as_slice(),
                            image.import.as_slice(),
                        ];

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
    fn mu_ns_interns(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_ns_externs(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Namespace {
    fn mu_untern(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let scope = fp.argv[1];
        let name = fp.argv[2];

        let ns = match Tag::type_of(mu, fp.argv[0]) {
            Type::Null => mu.unintern_ns,
            Type::Namespace => fp.argv[0],
            _ => return Err(Exception::new(Condition::Type, "mu:untern", fp.argv[0])),
        };

        let scope_type = match Tag::type_of(mu, scope) {
            Type::Keyword => {
                if scope.eq_(Symbol::keyword("extern")) {
                    Scope::Extern
                } else if scope.eq_(Symbol::keyword("intern")) {
                    Scope::Intern
                } else {
                    return Err(Exception::new(Condition::Type, "mu:untern", scope));
                }
            }
            _ => return Err(Exception::new(Condition::Type, "mu:untern", scope)),
        };

        fp.value = match Tag::type_of(mu, ns) {
            Type::Namespace => match Tag::type_of(mu, name) {
                Type::Vector if Vector::type_of(mu, name) == Type::Char => {
                    Self::intern(mu, ns, scope_type, Vector::as_string(mu, name), *UNBOUND)
                }
                _ => return Err(Exception::new(Condition::Type, "mu:untern", name)),
            },
            _ => return Err(Exception::new(Condition::Type, "mu:untern", ns)),
        };

        Ok(())
    }

    fn mu_intern(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let scope = fp.argv[1];
        let name = fp.argv[2];
        let value = fp.argv[3];

        let ns = match Tag::type_of(mu, fp.argv[0]) {
            Type::Null => mu.unintern_ns,
            Type::Namespace => fp.argv[0],
            _ => return Err(Exception::new(Condition::Type, "mu:intern", fp.argv[0])),
        };

        let scope_type = match Tag::type_of(mu, scope) {
            Type::Keyword => {
                if scope.eq_(Symbol::keyword("extern")) {
                    Scope::Extern
                } else if scope.eq_(Symbol::keyword("intern")) {
                    Scope::Intern
                } else {
                    return Err(Exception::new(Condition::Type, "mu:intern", scope));
                }
            }
            _ => return Err(Exception::new(Condition::Type, "mu:intern", scope)),
        };

        fp.value = match Tag::type_of(mu, ns) {
            Type::Namespace => match Tag::type_of(mu, name) {
                Type::Vector if Vector::type_of(mu, name) == Type::Char => {
                    Self::intern(mu, ns, scope_type, Vector::as_string(mu, name), value)
                }
                _ => return Err(Exception::new(Condition::Type, "mu:intern", name)),
            },
            _ => return Err(Exception::new(Condition::Type, "mu:intern", ns)),
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
                _ => Err(Exception::new(Condition::Type, "mu:make_ns", import)),
            },
            _ => Err(Exception::new(Condition::Type, "mu:make_ns", name)),
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
            _ => return Err(Exception::new(Condition::Type, "mu:map-ns", ns_name)),
        };

        Ok(())
    }

    fn mu_ns_find(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let scope = fp.argv[1];
        let name = fp.argv[2];

        let ns = match Tag::type_of(mu, fp.argv[0]) {
            Type::Null => mu.unintern_ns,
            Type::Namespace => fp.argv[0],
            _ => return Err(Exception::new(Condition::Type, "mu:intern", fp.argv[0])),
        };

        let scope = match Tag::type_of(mu, scope) {
            Type::Keyword => {
                if scope.eq_(Symbol::keyword("extern")) {
                    Scope::Extern
                } else if scope.eq_(Symbol::keyword("intern")) {
                    Scope::Intern
                } else {
                    return Err(Exception::new(Condition::Type, "mu:ns-find", scope));
                }
            }
            _ => return Err(Exception::new(Condition::Type, "mu:ns-find", scope)),
        };

        match Tag::type_of(mu, name) {
            Type::Vector => match Tag::type_of(mu, ns) {
                Type::Namespace => {
                    let ns_name = Vector::as_string(mu, Namespace::name(mu, ns));
                    let sy_name = Vector::as_string(mu, name);

                    fp.value = match <Mu as NSMaps>::map_ns(mu, &ns_name) {
                        Some(ns) => match <Mu as NSMaps>::map(mu, ns, scope, &sy_name) {
                            Some(sym) => sym,
                            None => Tag::nil(),
                        },
                        _ => return Err(Exception::new(Condition::Range, "mu:ns-find", name)),
                    };

                    Ok(())
                }
                _ => Err(Exception::new(Condition::Type, "mu:ns-find", ns)),
            },
            _ => Err(Exception::new(Condition::Type, "mu:ns-find", name)),
        }
    }

    fn mu_ns_import(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];

        match Tag::type_of(mu, ns) {
            Type::Namespace => {
                fp.value = Self::import(mu, ns);
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "mu:ns-ump", ns)),
        }
    }

    fn mu_ns_name(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];

        match Tag::type_of(mu, ns) {
            Type::Namespace => {
                fp.value = Self::name(mu, ns);
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "mu:ns-name", ns)),
        }
    }

    fn mu_ns_externs(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];

        match Tag::type_of(mu, ns) {
            Type::Namespace => {
                fp.value = Self::externs(mu, ns);
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "mu:ns-ext", ns)),
        }
    }

    fn mu_ns_interns(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ns = fp.argv[0];

        match Tag::type_of(mu, ns) {
            Type::Namespace => {
                fp.value = Self::interns(mu, ns);
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "mu:ns-int", ns)),
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
