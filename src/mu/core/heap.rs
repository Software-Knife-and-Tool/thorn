//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu gc
//!    Mu
#[allow(unused_imports)]
use {
    crate::{
        allocators::bump_allocator::BumpAllocator,
        core::{
            config::Config,
            direct::DirectTag,
            exception,
            frame::Frame,
            indirect::{self, IndirectTag},
            mu::{Core as _, Mu},
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
    },
    memmap,
    modular_bitfield::specifiers::{B11, B4},
    num_enum::TryFromPrimitive,
};

// locking protocols
use futures::executor::block_on;

#[derive(Debug, Copy, Clone)]
pub enum GcMode {
    None,
    Auto,
    Demand,
}

#[bitfield]
#[repr(align(8))]
#[derive(Debug, Copy, Clone)]
pub struct AllocImageInfo {
    pub reloc: u32, // relocation
    #[skip]
    __: B11, // expansion
    pub mark: bool, // reference counting
    pub len: u16,   // in bytes
    pub image_type: B4, // tag type
}

pub struct HeapAllocator<'a> {
    mmap: &'a memmap::MmapMut,

    alloc: fn(&HeapAllocator, &[[u8; 8]], u8) -> usize,
    #[allow(clippy::type_complexity)]
    valloc: fn(&HeapAllocator, &[[u8; 8]], &[u8], u8) -> usize,
    // begin_gc: fn(u8, u16) -> u32,

    /*
    info_iter: &'a dyn Iterator<Item = AllocImageInfo>,
    image_iter: &'a dyn Iterator<Item = AllocImageInfo>,
     */
    freelist: [Vec<Tag>; 16],
    page_size: usize,
    npages: usize,
}

pub enum AllocatorTypes {
    Bump(BumpAllocator),
}

pub struct Heap<'a> {
    allocator: HeapAllocator<'a>,
}

#[derive(Debug, Copy, Clone)]
pub struct AllocTypeInfo {
    pub size: usize,
    pub total: usize,
    pub free: usize,
}

pub trait Allocator {
    fn alloc(&mut self, _: &[[u8; 8]], _: Type) -> usize;
    fn valloc(&mut self, _: &[[u8; 8]], _: &[u8], _: Type) -> usize;
}

impl Allocator for Heap<'_> {
    fn alloc(&mut self, data: &[[u8; 8]], r#type: Type) -> usize {
        ((self.allocator).alloc)(&self.allocator, data, r#type as u8)
    }

    fn valloc(&mut self, data: &[[u8; 8]], vdata: &[u8], r#type: Type) -> usize {
        ((self.allocator).valloc)(&self.allocator, data, vdata, r#type as u8)
    }
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
    fn add_gc_root(_: &Mu, _: Tag);
    fn gc_asyncs(_: &Mu);
    fn gc_maps(_: &Mu);
    fn gc_namespaces(_: &Mu);
    fn mark(_: &Mu, _: Tag) -> Option<bool>;
    fn heap_size(_: &Mu, _: Tag) -> usize;
    fn heap_info(_: &Mu) -> (usize, usize);
    fn heap_type(_: &Mu, _: Type) -> AllocTypeInfo;
}

impl Core for Heap<'_> {
    fn add_gc_root(mu: &Mu, tag: Tag) {
        let mut root_ref = block_on(mu.gc_root.write());

        root_ref.push(tag);
    }

    fn mark(mu: &Mu, tag: Tag) -> Option<bool> {
        match tag {
            Tag::Direct(_) => None,
            Tag::Indirect(indirect) => {
                let mut heap_ref = block_on(mu.heap.write());

                let mark = heap_ref.get_image_refbit(indirect.image_id() as usize);
                heap_ref.set_image_refbit(indirect.image_id() as usize);

                mark
            }
        }
    }

    fn gc_asyncs(mu: &Mu) {
        let async_index_ref = block_on(mu.async_index.read());
        for (_name, _hash) in async_index_ref.iter() {
            // mu.gc_mark(*context)
        }
    }

    fn gc_maps(mu: &Mu) {
        let map_index_ref = block_on(mu.map_index.read());
        for (_name, hash) in map_index_ref.iter() {
            for (_key, map) in hash.iter() {
                mu.gc_mark(*map)
            }
        }
    }

    fn gc_namespaces(mu: &Mu) {
        let ns_index_ref = block_on(mu.ns_index.read());
        for (_name, hash) in ns_index_ref.iter() {
            // println!("1. marking {} namespace", Vector::as_string(mu, Symbol::name(mu, Tag::from_u64(*name))));
            let hash_ref = block_on(hash.1.read());
            for (_name, symbol) in hash_ref.iter() {
                // println!("    {}", _name);
                mu.gc_mark(*symbol)
            }
        }
    }

    fn heap_size(mu: &Mu, tag: Tag) -> usize {
        match tag.type_of() {
            Type::Cons => Cons::heap_size(mu, tag),
            Type::Function => Function::heap_size(mu, tag),
            Type::Map => Map::heap_size(mu, tag),
            Type::Stream => Stream::heap_size(mu, tag),
            Type::Struct => Struct::heap_size(mu, tag),
            Type::Symbol => Symbol::heap_size(mu, tag),
            Type::Vector => Vector::heap_size(mu, tag),
            _ => std::mem::size_of::<DirectTag>(),
        }
    }

    fn heap_info(mu: &Mu) -> (usize, usize) {
        let heap_ref = block_on(mu.heap.read());

        (heap_ref.page_size, heap_ref.npages)
    }

    fn heap_type(mu: &Mu, htype: Type) -> AllocTypeInfo {
        let heap_ref = block_on(mu.heap.read());
        let alloc_ref = block_on(heap_ref.alloc_map.read());
        let alloc_type = block_on(alloc_ref[htype as usize].read());

        *alloc_type
    }
}

pub trait MuFunction {
    fn mu_gc(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_hp_info(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_hp_size(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_hp_stat(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Heap<'_> {
    fn mu_gc(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match mu.gc() {
            Ok(_) => Symbol::keyword("t"),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_hp_stat(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let (pagesz, npages) = Heap::heap_info(mu);

        let mut vec = vec![
            Symbol::keyword("heap"),
            Fixnum::as_tag((pagesz * npages) as i64),
            Fixnum::as_tag(npages as i64),
            Fixnum::as_tag(0_i64),
        ];

        for htype in INFOTYPE.iter() {
            let type_map = Self::heap_type(
                mu,
                <IndirectTag as indirect::Core>::to_indirect_type(*htype).unwrap(),
            );

            vec.push(*htype);
            vec.push(Fixnum::as_tag(type_map.size as i64));
            vec.push(Fixnum::as_tag(type_map.total as i64));
            vec.push(Fixnum::as_tag(type_map.free as i64));
        }

        fp.value = TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu);
        Ok(())
    }

    fn mu_hp_info(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let (page_size, npages) = Self::heap_info(mu);

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
