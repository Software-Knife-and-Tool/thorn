//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu indirect
#![allow(unused_braces)]
#![allow(clippy::identity_op)]
use crate::{
    core::{
        exception,
        frame::Frame,
        mu::Mu,
        types::{Tag, TagType, Type},
    },
    modular_bitfield::specifiers::{B1, B60},
    types::{
        fixnum::Fixnum,
        symbol::{Core as _, Symbol},
        vecimage::{TypedVec, VecType},
        vector::Core as _,
    },
};

#[cfg(feature = "async")]
use futures::executor::block_on;

// little-endian tag format
#[derive(Copy, Clone)]
#[bitfield]
#[repr(u64)]
pub struct IndirectTag {
    #[bits = 3]
    pub tag: TagType,
    pub heap_id: B1,
    pub offset: B60,
}

lazy_static! {
    static ref TYPEMAP: Vec<(Tag, Type)> = vec![
        (Symbol::keyword("cons"), Type::Cons),
        (Symbol::keyword("func"), Type::Function),
        (Symbol::keyword("nil"), Type::Null),
        (Symbol::keyword("stream"), Type::Stream),
        (Symbol::keyword("symbol"), Type::Symbol),
        (Symbol::keyword("struct"), Type::Struct),
        (Symbol::keyword("t"), Type::T),
        (Symbol::keyword("vector"), Type::Vector),
    ];
    static ref INFOTYPE: Vec<Tag> = vec![
        Symbol::keyword("cons"),
        Symbol::keyword("func"),
        Symbol::keyword("stream"),
        Symbol::keyword("struct"),
        Symbol::keyword("symbol"),
        Symbol::keyword("vector"),
    ];
}

pub trait Core {
    fn to_type(_: Tag) -> Option<Type>;
    fn hp_info(_: &Mu) -> (usize, usize);
    fn hp_type(_: &Mu, _: Type) -> (u8, usize, usize, usize);
}

impl Core for Mu {
    fn to_type(keyword: Tag) -> Option<Type> {
        TYPEMAP
            .iter()
            .copied()
            .find(|tab| keyword.eq_(tab.0))
            .map(|tab| tab.1)
    }

    fn hp_info(mu: &Mu) -> (usize, usize) {
        #[cfg(feature = "async")]
        let heap_ref = block_on(mu.heap.read());
        #[cfg(not(feature = "async"))]
        let heap_ref = mu.heap.borrow();

        (heap_ref.page_size, heap_ref.npages)
    }

    fn hp_type(mu: &Mu, htype: Type) -> (u8, usize, usize, usize) {
        #[cfg(feature = "async")]
        let heap_ref = block_on(mu.heap.read());
        #[allow(clippy::type_complexity)]
        #[cfg(feature = "async")]
        let alloc_ref = block_on(heap_ref.alloc_map.read());

        #[cfg(not(feature = "async"))]
        let heap_ref = mu.heap.borrow();
        #[allow(clippy::type_complexity)]
        #[cfg(not(feature = "async"))]
        let alloc_ref = heap_ref.alloc_map.borrow();

        alloc_ref[htype as usize]
    }
}

pub trait MuFunction {
    fn mu_hp_info(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_hp_info(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let (pagesz, npages) = Self::hp_info(mu);

        let mut vec = vec![
            Symbol::keyword("t"),
            Fixnum::as_tag((pagesz * npages) as i64),
            Fixnum::as_tag(npages as i64),
            Fixnum::as_tag(npages as i64),
        ];

        for htype in INFOTYPE.iter() {
            let (_, size, alloc, in_use) = Self::hp_type(mu, Self::to_type(*htype).unwrap());

            vec.push(*htype);
            vec.push(Fixnum::as_tag(size as i64));
            vec.push(Fixnum::as_tag(alloc as i64));
            vec.push(Fixnum::as_tag(in_use as i64));
        }

        fp.value = TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn image() {
        assert_eq!(2 + 2, 4);
    }
}
