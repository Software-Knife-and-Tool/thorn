//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu gc
//!    Mu
use crate::{
    core::{
        direct::DirectTag,
        exception,
        frame::Frame,
        indirect::{self, IndirectTag},
        mu::Mu,
        types::{Tag, Type},
    },
    types::{
        char::{Char, Core as _},
        cons::{Cons, Core as _},
        fixnum::{Core as _, Fixnum},
        float::{Core as _, Float},
        function::{Core as _, Function},
        map::{Core as _, Map},
        stream::{Core as _, Stream},
        struct_::{Core as _, Struct},
        symbol::{Core as _, Symbol},
        vecimage::{TypedVec, VecType},
        vector::{Core as _, Vector},
    },
};

// locking protocols
use futures::executor::block_on;

lazy_static! {
    static ref INFOTYPE: Vec<Tag> = vec![
        Symbol::keyword("cons"),
        Symbol::keyword("func"),
        Symbol::keyword("map"),
        Symbol::keyword("stream"),
        Symbol::keyword("struct"),
        Symbol::keyword("symbol"),
        Symbol::keyword("vector"),
    ];
}

pub trait Core {
    fn add_gc_root(&self, _: Tag);
    fn gc(&self) -> exception::Result<bool>;
    fn gc_mark(&self, _: Tag);
    fn heap_size(&self, _: Tag) -> usize;
    fn heap_info(_: &Mu) -> (usize, usize);
    fn heap_type(_: &Mu, _: Type) -> (u8, usize, usize, usize);
}

impl Core for Mu {
    fn add_gc_root(&self, tag: Tag) {
        let mut root_ref = block_on(self.gc_root.write());

        root_ref.push(tag);
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

        Ok(true)
    }

    fn heap_size(&self, tag: Tag) -> usize {
        match Tag::type_of(tag) {
            Type::Cons => Cons::heap_size(self, tag),
            Type::Function => Function::heap_size(self, tag),
            Type::Map => Map::heap_size(self, tag),
            Type::Stream => Stream::heap_size(self, tag),
            Type::Struct => Struct::heap_size(self, tag),
            Type::Symbol => Symbol::heap_size(self, tag),
            Type::Vector => Vector::heap_size(self, tag),
            _ => std::mem::size_of::<DirectTag>(),
        }
    }

    fn heap_info(mu: &Mu) -> (usize, usize) {
        let heap_ref = block_on(mu.heap.read());

        (heap_ref.page_size, heap_ref.npages)
    }

    fn heap_type(mu: &Mu, htype: Type) -> (u8, usize, usize, usize) {
        let heap_ref = block_on(mu.heap.read());
        let alloc_ref = block_on(heap_ref.alloc_map.read());

        alloc_ref[htype as usize]
    }
}

pub trait MuFunction {
    fn mu_gc(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_hp_info(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_hp_size(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_view(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_hp_info(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let (pagesz, npages) = Self::heap_info(mu);

        let mut vec = vec![
            Symbol::keyword("t"),
            Fixnum::as_tag((pagesz * npages) as i64),
            Fixnum::as_tag(npages as i64),
            Fixnum::as_tag(npages as i64),
        ];

        for htype in INFOTYPE.iter() {
            let (_, size, alloc, in_use) = Self::heap_type(
                mu,
                <IndirectTag as indirect::Core>::to_indirect_type(*htype).unwrap(),
            );

            vec.push(*htype);
            vec.push(Fixnum::as_tag(size as i64));
            vec.push(Fixnum::as_tag(alloc as i64));
            vec.push(Fixnum::as_tag(in_use as i64));
        }

        fp.value = TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu);
        Ok(())
    }

    fn mu_gc(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match mu.gc() {
            Ok(_) => Symbol::keyword("t"),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_hp_size(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = Fixnum::as_tag(Self::heap_size(mu, fp.argv[0]) as i64);

        Ok(())
    }

    fn mu_view(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let tag = fp.argv[0];

        fp.value = match Tag::type_of(tag) {
            Type::Char => Char::view(mu, tag),
            Type::Cons => Cons::view(mu, tag),
            Type::Fixnum => Fixnum::view(mu, tag),
            Type::Float => Float::view(mu, tag),
            Type::Function => Function::view(mu, tag),
            Type::Map => Map::view(mu, tag),
            Type::Stream => Stream::view(mu, tag),
            Type::Struct => Struct::view(mu, tag),
            Type::Vector => Vector::view(mu, tag),
            _ => Symbol::view(mu, tag),
        };

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
