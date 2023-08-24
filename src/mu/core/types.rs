//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu tagged types
#![allow(unused_braces)]
#![allow(clippy::identity_op)]
use {
    crate::{
        core::{
            cdirect::ConsDirectTag,
            direct::{DirectTag, DirectType},
            exception,
            frame::Frame,
            indirect::IndirectTag,
            mu::Mu,
        },
        types::symbol::{Core as _, Symbol},
    },
    num_enum::TryFromPrimitive,
    std::fmt,
};

// tag storage classes
#[derive(Copy, Clone)]
pub enum Tag {
    Fixnum(i64),
    ConsDirect(ConsDirectTag),
    Direct(DirectTag),
    Indirect(IndirectTag),
}

// types
#[derive(PartialEq, Copy, Clone, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Type {
    Byte,
    Char,
    Cons,
    Fixnum,
    Float,
    Function,
    Keyword,
    Namespace,
    Null,
    Stream,
    Struct,
    Symbol,
    T,
    Vector,
}

#[derive(BitfieldSpecifier, Copy, Clone, Debug, PartialEq, Eq)]
pub enum TagType {
    Fixnum = 0,     // 61 bit signed integer
    ConsDirect = 1, // cons direct
    Direct = 2,     // chars, short strings, keywords
    Indirect = 3,   // heap tags
    Cons = 4,       // cons heap tag
    Function = 5,   // function heap tag
    Symbol = 6,     // symbol heap tag
    Vector = 7,     // vector heap tag
}

lazy_static! {
    static ref NIL: Tag = Tag::to_direct(
        (('l' as u64) << 16) | (('i' as u64) << 8) | ('n' as u64),
        3,
        DirectType::Keyword
    );
    pub static ref TYPEKEYMAP: Vec::<(Type, Tag)> = vec![
        (Type::Byte, Symbol::keyword("byte")),
        (Type::Char, Symbol::keyword("char")),
        (Type::Cons, Symbol::keyword("cons")),
        (Type::Fixnum, Symbol::keyword("fixnum")),
        (Type::Float, Symbol::keyword("float")),
        (Type::Function, Symbol::keyword("func")),
        (Type::Keyword, Symbol::keyword("keyword")),
        (Type::Namespace, Symbol::keyword("ns")),
        (Type::Null, Symbol::keyword("null")),
        (Type::Stream, Symbol::keyword("stream")),
        (Type::Struct, Symbol::keyword("struct")),
        (Type::Symbol, Symbol::keyword("symbol")),
        (Type::T, Symbol::keyword("t")),
        (Type::Vector, Symbol::keyword("vector")),
    ];
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}: ", self.as_u64()).unwrap();
        match self {
            Tag::Fixnum(i64_) => write!(f, "fixnum: {}", i64_ >> 3),
            Tag::Direct(direct) => write!(f, "direct: type {:?}", direct.dtype() as u8),
            Tag::Indirect(indirect) => write!(f, "indirect: type {:?}", indirect.tag()),
            Tag::ConsDirect(_) => write!(f, "cons-direct:"),
        }
    }
}

impl Tag {
    pub fn data(&self, mu: &Mu) -> u64 {
        let heap_ref = mu.heap.read().unwrap();
        match self {
            Tag::Fixnum(fx) => (*fx >> 3) as u64,
            Tag::Direct(tag) => tag.data(),
            Tag::Indirect(heap) => match heap_ref.info(heap.offset() as usize) {
                Some(info) => match Type::try_from(info.tag_type()) {
                    Ok(etype) => etype as u64,
                    Err(_) => panic!(),
                },
                None => panic!(),
            },
            Tag::ConsDirect(tag) => u64::from(tag.car() << 30 | tag.cdr()),
        }
    }

    pub fn as_slice(&self) -> [u8; 8] {
        match self {
            Tag::ConsDirect(tag) => tag.into_bytes(),
            Tag::Direct(tag) => tag.into_bytes(),
            Tag::Fixnum(tag) => tag.to_le_bytes(),
            Tag::Indirect(tag) => tag.into_bytes(),
        }
    }

    pub fn eq_(&self, tag: Tag) -> bool {
        self.as_u64() == tag.as_u64()
    }

    pub fn null_(&self) -> bool {
        self.eq_(Self::nil())
    }

    pub fn nil() -> Tag {
        *NIL
    }

    pub fn as_u64(&self) -> u64 {
        u64::from_le_bytes(self.as_slice())
    }

    pub fn from_u64(tag: u64) -> Tag {
        Self::from_slice(&tag.to_le_bytes())
    }

    pub fn from_slice(bits: &[u8]) -> Tag {
        let mut data: [u8; 8] = 0u64.to_le_bytes();
        for (src, dst) in bits.iter().zip(data.iter_mut()) {
            *dst = *src
        }

        let tag: u8 = (u64::from_le_bytes(data) & 0x7) as u8;
        let _u64: u64 = u64::from_le_bytes(data);

        match tag {
            tag if tag == TagType::Fixnum as u8 => Tag::Fixnum(_u64 as i64),
            tag if tag == TagType::ConsDirect as u8 => Tag::ConsDirect(ConsDirectTag::from(_u64)),
            tag if tag == TagType::Direct as u8 => Tag::Direct(DirectTag::from(_u64)),
            _ => Tag::Indirect(IndirectTag::from(_u64)),
        }
    }

    pub fn type_of(mu: &Mu, tag: Tag) -> Type {
        let heap_ref = mu.heap.read().unwrap();

        if tag.null_() {
            Type::Null
        } else {
            match tag {
                Tag::Fixnum(_) => Type::Fixnum,
                Tag::ConsDirect(_) => Type::Cons,
                Tag::Direct(direct) => match direct.dtype() {
                    DirectType::Char => Type::Char,
                    DirectType::Byte => Type::Vector,
                    DirectType::Keyword => Type::Keyword,
                    DirectType::Float => Type::Float,
                },
                Tag::Indirect(indirect) => match indirect.tag() {
                    TagType::Symbol => Type::Symbol,
                    TagType::Function => Type::Function,
                    TagType::Cons => Type::Cons,
                    TagType::Vector => Type::Vector,
                    TagType::Indirect => match heap_ref.info(indirect.offset() as usize) {
                        Some(info) => match Type::try_from(info.tag_type()) {
                            Ok(etype) => etype,
                            Err(_) => panic!(),
                        },
                        None => panic!(),
                    },
                    _ => panic!("indirect type-of botch {:x}", tag.as_u64()),
                },
            }
        }
    }

    pub fn type_key(ttype: Type) -> Option<Tag> {
        TYPEKEYMAP
            .iter()
            .copied()
            .find(|map| ttype == map.0)
            .map(|map| map.1)
    }

    pub fn key_type(tag: Tag) -> Option<Type> {
        TYPEKEYMAP
            .iter()
            .copied()
            .find(|map| tag.eq_(map.1))
            .map(|map| map.0)
    }
}

pub trait MuFunction {
    fn mu_eq(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_typeof(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Tag {
    fn mu_eq(_: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = if fp.argv[0].eq_(fp.argv[1]) {
            Symbol::keyword("t")
        } else {
            Tag::nil()
        };

        Ok(())
    }

    fn mu_typeof(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match Self::type_key(Self::type_of(mu, fp.argv[0])) {
            Some(type_key) => type_key,
            None => panic!(),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn types() {
        assert_eq!(2 + 2, 4);
    }
}
