//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu gc
//!    Mu
#[allow(unused_imports)]
use {
    crate::{
        core::{
            config::Config,
            direct::DirectTag,
            exception,
            frame::Frame,
            indirect::{self, IndirectTag},
            mu::{Core as _, Mu},
            types::{Tag, Type},
        },
        heap::bump_heap::BumpHeap,
        types::{
            char::{Char, Core as _},
            cons::{Cons, Core as _},
            fixnum::{Core as _, Fixnum},
            float::{Core as _, Float},
            function::{Core as _, Function},
            map::{Core as _, Map},
            r#struct::{Core as _, Struct},
            stream::{Core as _, Stream},
            symbol::{Core as _, Symbol},
            vecimage::{TypedVec, VecType},
            vector::{Core as _, Vector},
        },
    },
    memmap,
    modular_bitfield::specifiers::{B11, B4},
};

// locking protocols
use {futures::executor::block_on, futures_locks::RwLock};

#[derive(Copy, Clone)]
pub enum GcMode {
    None,
    Auto,
    Demand,
}

// (type, total-size, alloc, in-use)
type AllocMap = (u8, usize, usize, usize);

#[bitfield]
#[repr(align(8))]
#[derive(Debug, Copy, Clone)]
pub struct Info {
    pub reloc: u32, // relocation
    #[skip]
    __: B11, // expansion
    pub mark: bool, // reference counting
    pub len: u16,   // in bytes
    pub image_type: B4, // tag type
}

pub struct HeapInterface<'a> {
    mmap: &'a memmap::MmapMut,
    alloc: Box<dyn Fn(u8, u16) -> u32>,
    begin_gc: Box<dyn Fn(u8, u16) -> u32>,
    info_iter: Box<dyn Iterator<Item = Info>>,
    image_iter: Box<dyn Iterator<Item = Info>>,
    page_size: usize,
    npages: usize,
}

pub struct Heap {
    heap: HeapInterface<'static>,
    alloc_map: RwLock<Vec<AllocMap>>,
    freelist: [Vec<Tag>; 16],
}

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
    fn mark(&self, _: Tag) -> Option<bool>;
    fn heap_size(&self, _: Tag) -> usize;
    fn heap_info(_: &Mu) -> (usize, usize);
    fn heap_type(_: &Mu, _: Type) -> (u8, usize, usize, usize);
}

impl Core for Mu {
    fn add_gc_root(&self, tag: Tag) {
        let mut root_ref = block_on(self.gc_root.write());

        root_ref.push(tag);
    }

    fn mark(&self, tag: Tag) -> Option<bool> {
        match tag {
            Tag::Direct(_) => None,
            Tag::Indirect(indirect) => {
                let mut heap_ref = block_on(self.heap.write());

                let mark = heap_ref.get_image_refbit(indirect.image_id() as usize);
                heap_ref.set_image_refbit(indirect.image_id() as usize);

                mark
            }
        }
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
    fn mu_hp_stat(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_view(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_gc(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match mu.gc() {
            Ok(_) => Symbol::keyword("t"),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_hp_stat(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
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

    fn mu_hp_info(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let (page_size, npages) = Self::heap_info(mu);

        {
            let heap_ref = block_on(mu.heap.read());

            heap_ref.gc_stats()
        }

        let vec = vec![
            Symbol::keyword("bump"),
            Fixnum::as_tag(page_size as i64),
            Fixnum::as_tag(npages as i64),
        ];

        fp.value = TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu);

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

/*
/// iterator
pub struct HeapInfoIter<'a> {
    pub heap: &'a Heap,
    pub offset: usize,
}

impl<'a> HeapInfoIter<'a> {
    pub fn new(heap: &'a Heap) -> Self {
        Self { heap, offset: 8 }
    }
}

impl<'a> Iterator for HeapInfoIter<'a> {
    type Item = Info;

    fn next(&mut self) -> Option<Self::Item> {
        let info = self.heap.image_info(self.offset).unwrap();
        self.offset += info.len() as usize;

        Some(info)
    }
}
*/
