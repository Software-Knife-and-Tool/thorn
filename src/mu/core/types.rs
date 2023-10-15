//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu tagged types
#![allow(unused_braces)]
#![allow(clippy::identity_op)]
use {
    crate::{
        core::{
            direct::{DirectInfo, DirectTag, DirectType, ExtType},
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

use futures::executor::block_on;

// tag storage classes
#[derive(Copy, Clone)]
pub enum Tag {
    Direct(DirectTag),
    Indirect(IndirectTag),
}

// types
#[derive(PartialEq, Copy, Clone, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Type {
    AsyncId,
    Byte,
    Char,
    Cons,
    Fixnum,
    Float,
    Function,
    Keyword,
    Map,
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
    Direct = 0,   // 56 bit direct objects
    Cons = 1,     // cons heap tag
    Function = 2, // function heap tag
    Stream = 3,   // stream heap tag
    Struct = 4,   // struct heap tags
    Symbol = 5,   // symbol heap tag
    Vector = 6,   // vector heap tag
    Map = 7,      // map vector tag
}

lazy_static! {
    static ref NIL: Tag = DirectTag::to_direct(
        (('l' as u64) << 16) | (('i' as u64) << 8) | ('n' as u64),
        DirectInfo::Length(3),
        DirectType::Keyword
    );
    pub static ref TYPEKEYMAP: Vec::<(Type, Tag)> = vec![
        (Type::AsyncId, Symbol::keyword("asyncid")),
        (Type::Byte, Symbol::keyword("byte")),
        (Type::Char, Symbol::keyword("char")),
        (Type::Cons, Symbol::keyword("cons")),
        (Type::Fixnum, Symbol::keyword("fixnum")),
        (Type::Float, Symbol::keyword("float")),
        (Type::Function, Symbol::keyword("func")),
        (Type::Keyword, Symbol::keyword("keyword")),
        (Type::Map, Symbol::keyword("map")),
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
            Tag::Direct(direct) => write!(f, "direct: type {:?}", direct.dtype() as u8),
            Tag::Indirect(indirect) => write!(f, "indirect: type {:?}", indirect.tag()),
        }
    }
}

impl Tag {
    pub fn data(&self, mu: &Mu) -> u64 {
        let heap_ref = block_on(mu.heap.read());

        match self {
            Tag::Direct(tag) => tag.data(),
            Tag::Indirect(heap) => match heap_ref.info(heap.offset() as usize) {
                Some(info) => match Type::try_from(info.tag_type()) {
                    Ok(etype) => etype as u64,
                    Err(_) => panic!(),
                },
                None => panic!(),
            },
        }
    }

    pub fn as_slice(&self) -> [u8; 8] {
        match self {
            Tag::Direct(tag) => tag.into_bytes(),
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
            tag if tag == TagType::Direct as u8 => Tag::Direct(DirectTag::from(_u64)),
            _ => Tag::Indirect(IndirectTag::from(_u64)),
        }
    }

    pub fn type_of(tag: Tag) -> Type {
        if tag.null_() {
            Type::Null
        } else {
            match tag {
                Tag::Direct(direct) => match direct.dtype() {
                    DirectType::Byte => Type::Vector,
                    DirectType::Char => Type::Char,
                    DirectType::Keyword => Type::Keyword,
                    DirectType::Ext => match ExtType::try_from(direct.info()) {
                        Ok(ExtType::Float) => Type::Float,
                        Ok(ExtType::AsyncId) => Type::AsyncId,
                        Ok(ExtType::Cons) => Type::Cons,
                        Ok(ExtType::Fixnum) => Type::Fixnum,
                        _ => panic!("direct type botch {:x}", tag.as_u64()),
                    },
                },
                Tag::Indirect(indirect) => match indirect.tag() {
                    TagType::Cons => Type::Cons,
                    TagType::Function => Type::Function,
                    TagType::Stream => Type::Stream,
                    TagType::Struct => Type::Struct,
                    TagType::Symbol => Type::Symbol,
                    TagType::Vector => Type::Vector,
                    TagType::Map => Type::Map,
                    _ => panic!("indirect type botch {:x}", tag.as_u64()),
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

    fn mu_typeof(_: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match Self::type_key(Self::type_of(fp.argv[0])) {
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
