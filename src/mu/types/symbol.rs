//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu symbol type
use {
    crate::{
        core::{
            direct::DirectType,
            exception::{self, Condition, Exception},
            frame::Frame,
            indirect::IndirectTag,
            mu::{Core as _, Mu},
            namespace::{Core as _, NSMaps, Namespace},
            readtable::{map_char_syntax, SyntaxType},
            types::{Tag, TagType, Type},
        },
        types::{
            char::Char,
            stream::{Core as _, Stream},
            vecimage::{TypedVec, VecType},
            vector::{Core as _, Vector},
        },
    },
    std::str,
};

pub enum Symbol {
    Keyword(Tag),
    Symbol(SymbolImage),
}

pub struct SymbolImage {
    pub namespace: Tag,
    pub name: Tag,
    pub value: Tag,
}

lazy_static! {
    pub static ref UNBOUND: Tag = Tag::to_direct(0, 0, DirectType::Keyword);
}

impl Symbol {
    pub fn new(mu: &Mu, namespace: Tag, name: &str, value: Tag) -> Self {
        let str = name.as_bytes();

        match str[0] as char {
            ':' => Symbol::Keyword(Self::keyword(&name[1..])),
            _ => Symbol::Symbol(SymbolImage {
                namespace,
                name: Vector::from_string(name).evict(mu),
                value,
            }),
        }
    }

    pub fn to_image(mu: &Mu, tag: Tag) -> SymbolImage {
        let heap_ref = mu.heap.read().unwrap();
        match Tag::type_of(mu, tag) {
            Type::Symbol => match tag {
                Tag::Indirect(main) => SymbolImage {
                    namespace: Tag::from_slice(
                        heap_ref.of_length(main.offset() as usize, 8).unwrap(),
                    ),
                    name: Tag::from_slice(
                        heap_ref.of_length(main.offset() as usize + 8, 8).unwrap(),
                    ),
                    value: Tag::from_slice(
                        heap_ref.of_length(main.offset() as usize + 16, 8).unwrap(),
                    ),
                },
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn namespace(mu: &Mu, symbol: Tag) -> Tag {
        match Tag::type_of(mu, symbol) {
            Type::Keyword => mu.keyword_ns,
            Type::Symbol => Self::to_image(mu, symbol).namespace,
            _ => panic!(),
        }
    }

    pub fn name(mu: &Mu, symbol: Tag) -> Tag {
        match Tag::type_of(mu, symbol) {
            Type::Null | Type::Keyword => match symbol {
                Tag::Direct(dir) => Tag::to_direct(dir.data(), dir.length(), DirectType::Byte),
                _ => panic!(),
            },
            Type::Symbol => Self::to_image(mu, symbol).name,
            _ => panic!(),
        }
    }

    pub fn value(mu: &Mu, symbol: Tag) -> Tag {
        match Tag::type_of(mu, symbol) {
            Type::Keyword => symbol,
            Type::Symbol => Self::to_image(mu, symbol).value,
            _ => panic!(),
        }
    }
}

pub trait Core {
    fn evict(&self, _: &Mu) -> Tag;
    fn is_unbound(_: &Mu, _: Tag) -> bool;
    fn keyword(_: &str) -> Tag;
    fn parse(_: &Mu, _: String) -> exception::Result<Tag>;
    fn view(_: &Mu, _: Tag) -> Tag;
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
}

impl Core for Symbol {
    fn view(mu: &Mu, symbol: Tag) -> Tag {
        let vec = vec![
            Self::namespace(mu, symbol),
            Self::name(mu, symbol),
            if Self::is_unbound(mu, symbol) {
                Symbol::keyword("UNBOUND")
            } else {
                Self::value(mu, symbol)
            },
        ];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn evict(&self, mu: &Mu) -> Tag {
        match self {
            Symbol::Keyword(tag) => *tag,
            Symbol::Symbol(image) => {
                let slices: &[[u8; 8]] = &[
                    image.namespace.as_slice(),
                    image.name.as_slice(),
                    image.value.as_slice(),
                ];

                let mut heap_ref = mu.heap.write().unwrap();
                Tag::Indirect(
                    IndirectTag::new()
                        .with_offset(heap_ref.alloc(slices, Type::Symbol as u8) as u64)
                        .with_heap_id(1)
                        .with_tag(TagType::Symbol),
                )
            }
        }
    }

    fn keyword(name: &str) -> Tag {
        let str = name.as_bytes();
        let len = str.len();

        if len > Tag::DIRECT_STR_MAX || len == 0 {
            panic!("{} {:?}", std::str::from_utf8(str).unwrap(), str)
        }

        let str = name.to_string();
        let mut data: [u8; 8] = 0u64.to_le_bytes();
        for (src, dst) in str.as_bytes().iter().zip(data.iter_mut()) {
            *dst = *src
        }
        Tag::to_direct(u64::from_le_bytes(data), len as u8, DirectType::Keyword)
    }

    fn parse(mu: &Mu, token: String) -> exception::Result<Tag> {
        for ch in token.chars() {
            match map_char_syntax(ch) {
                Some(SyntaxType::Constituent) => (),
                _ => {
                    return Err(Exception::new(Condition::Range, "symbol", Char::as_tag(ch)));
                }
            }
        }

        match token.find(':') {
            Some(0) => {
                if token.starts_with(':')
                    && (token.len() > Tag::DIRECT_STR_MAX + 1 || token.len() == 1)
                {
                    return Err(Exception::new(
                        Condition::Syntax,
                        "read:sy",
                        Vector::from_string(&token).evict(mu),
                    ));
                }
                Ok(Symbol::new(mu, Tag::nil(), &token, *UNBOUND).evict(mu))
            }
            Some(_) => {
                let sym: Vec<&str> = token.split(':').collect();
                let ns = sym[0].to_string();
                let name = sym[1].to_string();

                if sym.len() != 2 {
                    return Err(Exception::new(
                        Condition::Syntax,
                        "read:sy",
                        Vector::from_string(&token).evict(mu),
                    ));
                }

                match Mu::map_ns(mu, &ns) {
                    Some(ns) => Ok(Namespace::intern(mu, ns, name, *UNBOUND)),
                    None => Err(Exception::new(
                        Condition::Namespace,
                        "read:sy",
                        Vector::from_string(sym[0]).evict(mu),
                    )),
                }
            }
            None => Ok(Namespace::intern(mu, mu.null_ns, token, *UNBOUND)),
        }
    }

    fn write(mu: &Mu, symbol: Tag, escape: bool, stream: Tag) -> exception::Result<()> {
        match Tag::type_of(mu, symbol) {
            Type::Null | Type::Keyword => match str::from_utf8(&symbol.data(mu).to_le_bytes()) {
                Ok(s) => {
                    Stream::write_char(mu, stream, ':').unwrap();
                    for nth in 0..symbol.length() {
                        match Stream::write_char(mu, stream, s.as_bytes()[nth as usize] as char) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                    }
                    Ok(())
                }
                Err(_) => panic!(),
            },
            Type::Symbol => {
                let name = Self::name(mu, symbol);

                if escape {
                    let ns = Self::namespace(mu, symbol);

                    if !Tag::null_(&ns) && !mu.null_ns.eq_(ns) {
                        match mu.write(Namespace::name(mu, ns), false, stream) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }

                        match mu.write_string(":".to_string(), stream) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                    }
                }
                mu.write(name, false, stream)
            }
            _ => panic!(),
        }
    }

    fn is_unbound(mu: &Mu, symbol: Tag) -> bool {
        Self::value(mu, symbol).eq_(*UNBOUND)
    }
}

pub trait MuFunction {
    fn mu_name(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_ns(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_value(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_boundp(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_symbol(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_keyword(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Symbol {
    fn mu_name(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let symbol = fp.argv[0];

        fp.value = match Tag::type_of(mu, symbol) {
            Type::Null | Type::Keyword | Type::Symbol => Symbol::name(mu, symbol),
            _ => return Err(Exception::new(Condition::Type, "sy:name", symbol)),
        };

        Ok(())
    }

    fn mu_ns(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let symbol = fp.argv[0];

        fp.value = match Tag::type_of(mu, symbol) {
            Type::Symbol | Type::Keyword => Symbol::namespace(mu, symbol),
            _ => return Err(Exception::new(Condition::Type, "sy:ns", symbol)),
        };

        Ok(())
    }

    fn mu_value(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let symbol = fp.argv[0];

        fp.value = match Tag::type_of(mu, symbol) {
            Type::Symbol => {
                if Symbol::is_unbound(mu, symbol) {
                    return Err(Exception::new(Condition::Type, "sy-val", symbol));
                } else {
                    Symbol::value(mu, symbol)
                }
            }
            Type::Keyword => symbol,
            _ => return Err(Exception::new(Condition::Type, "sy-ns", symbol)),
        };

        Ok(())
    }

    fn mu_boundp(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let symbol = fp.argv[0];

        fp.value = match Tag::type_of(mu, symbol) {
            Type::Keyword => symbol,
            Type::Symbol => {
                if Self::is_unbound(mu, symbol) {
                    Tag::nil()
                } else {
                    symbol
                }
            }
            _ => return Err(Exception::new(Condition::Type, "unboundp", symbol)),
        };

        Ok(())
    }

    fn mu_keyword(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let symbol = fp.argv[0];

        match Tag::type_of(mu, symbol) {
            Type::Keyword => {
                fp.value = symbol;
                Ok(())
            }
            Type::Vector => {
                let str = Vector::as_string(mu, symbol);
                fp.value = Self::keyword(&str);
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "keyword", symbol)),
        }
    }

    fn mu_symbol(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let symbol = fp.argv[0];

        match Tag::type_of(mu, symbol) {
            Type::Vector => {
                let str = Vector::as_string(mu, symbol);
                fp.value = Self::new(mu, Tag::nil(), &str, *UNBOUND).evict(mu);
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "make-sy", symbol)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
