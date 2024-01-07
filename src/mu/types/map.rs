//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu maps
use {
    crate::{
        core::{
            exception::{self, Condition, Exception},
            frame::Frame,
            funcall::Core as _,
            heap::{self, Core as _},
            indirect::IndirectTag,
            mu::{Core as _, Mu},
            stream,
            types::{Tag, TagType, Type},
        },
        types::{
            cons::{Cons, ConsIter},
            fixnum::Fixnum,
            symbol::{Core as _, Symbol},
            vecimage::{TypedVec, VecType},
            vector::Core as _,
        },
    },
    std::collections::HashMap,
};

use futures::executor::block_on;

#[derive(Copy, Clone)]
pub struct Map {
    cache_id: Tag, // cache id, fixnum
    list: Tag,     // list of pairs
}

impl Map {
    fn new(mu: &Mu, list: Tag) -> Self {
        let mut index_ref = block_on(mu.map_index.write());
        let cache_id = index_ref.len();
        let mut map = HashMap::<u64, Tag>::new();

        for cons in ConsIter::new(mu, list) {
            let pair = Cons::car(mu, cons);

            map.insert(Tag::as_u64(&Cons::car(mu, pair)), Cons::cdr(mu, pair));
        }

        index_ref.insert(cache_id, map);

        Map {
            cache_id: Fixnum::as_tag(cache_id as i64),
            list,
        }
    }

    fn to_image(mu: &Mu, tag: Tag) -> Self {
        match tag.type_of() {
            Type::Map => match tag {
                Tag::Indirect(main) => {
                    let heap_ref = block_on(mu.heap.read());

                    Map {
                        cache_id: Tag::from_slice(
                            heap_ref.image_slice(main.image_id() as usize, 8).unwrap(),
                        ),
                        list: Tag::from_slice(
                            heap_ref
                                .image_slice(main.image_id() as usize + 8, 8)
                                .unwrap(),
                        ),
                    }
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    fn cache_id(mu: &Mu, map: Tag) -> Tag {
        Self::to_image(mu, map).cache_id
    }

    fn list(mu: &Mu, map: Tag) -> Tag {
        Self::to_image(mu, map).list
    }

    fn view(mu: &Mu, map: Tag) -> Tag {
        let image = Self::to_image(mu, map);
        let vec = vec![image.cache_id, image.list];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn heap_size(_: &Mu, _: Tag) -> usize {
        std::mem::size_of::<Map>()
    }

    fn map_ref(mu: &Mu, cache_id: usize, key: Tag) -> Option<Tag> {
        let index_ref = block_on(mu.map_index.read());

        match index_ref.get(&cache_id) {
            Some(hash) => hash.get(&key.as_u64()).copied(),
            None => None,
        }
    }

    fn evict(&self, mu: &Mu) -> Tag {
        let image: &[[u8; 8]] = &[self.cache_id.as_slice(), self.list.as_slice()];

        let mut heap_ref = block_on(mu.heap.write());
        let ind = IndirectTag::new()
            .with_image_id(heap_ref.alloc(image, Type::Map as u8) as u64)
            .with_heap_id(1)
            .with_tag(TagType::Map);

        Tag::Indirect(ind)
    }
}

pub trait Core {
    fn evict(&self, _: &Mu) -> Tag;
    fn gc_mark(_: &Mu, _: Tag);
    fn heap_size(_: &Mu, _: Tag) -> usize;
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn view(_: &Mu, _: Tag) -> Tag;
}

impl Core for Map {
    fn gc_mark(mu: &Mu, map: Tag) {
        let mark = mu.mark(map).unwrap();

        if !mark {
            mu.gc_mark(Self::list(mu, map))
        }
    }

    fn view(mu: &Mu, map: Tag) -> Tag {
        let vec = vec![Self::cache_id(mu, map), Self::list(mu, map)];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn heap_size(mu: &Mu, symbol: Tag) -> usize {
        std::mem::size_of::<Map>() + heap::Core::heap_size(mu, Self::list(mu, symbol))
    }

    fn write(mu: &Mu, map: Tag, _: bool, stream: Tag) -> exception::Result<()> {
        match map.type_of() {
            Type::Map => {
                let cache_id = Fixnum::as_i64(Self::cache_id(mu, map));
                let index_ref = block_on(mu.map_index.write());
                let size = index_ref.len();

                <Mu as stream::Core>::write_string(
                    mu,
                    format!("#<:map [size:{size}, tag:{}]>", cache_id).as_str(),
                    stream,
                )
            }
            _ => panic!(),
        }
    }

    fn evict(&self, mu: &Mu) -> Tag {
        let image: &[[u8; 8]] = &[self.cache_id.as_slice(), self.list.as_slice()];

        let mut heap_ref = block_on(mu.heap.write());
        let ind = IndirectTag::new()
            .with_image_id(heap_ref.alloc(image, Type::Map as u8) as u64)
            .with_heap_id(1)
            .with_tag(TagType::Function);

        Tag::Indirect(ind)
    }
}

pub trait MuFunction {
    fn mu_make_map(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_map_has(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_map_items(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_map_ref(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_map_size(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_make_map(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let list = fp.argv[0];

        fp.value = match mu.fp_argv_check("map", &[Type::List], fp) {
            Ok(_) => {
                for cons in ConsIter::new(mu, list) {
                    if Cons::car(mu, cons).type_of() != Type::Cons {
                        return Err(Exception::new(Condition::Type, "map", Cons::car(mu, cons)));
                    }
                }

                Map::new(mu, list).evict(mu)
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_map_ref(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let map = fp.argv[0];
        let key = fp.argv[1];

        fp.value = match mu.fp_argv_check("mp-ref", &[Type::Map, Type::T], fp) {
            Ok(_) => {
                let cache_id = Map::cache_id(mu, map);

                match Map::map_ref(mu, Fixnum::as_i64(cache_id) as usize, key) {
                    Some(value) => value,
                    None => return Err(Exception::new(Condition::Range, "mp-ref", key)),
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_map_has(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let map = fp.argv[0];
        let key = fp.argv[1];

        fp.value = match mu.fp_argv_check("mp-has", &[Type::Map, Type::T], fp) {
            Ok(_) => {
                let cache_id = Map::cache_id(mu, map);

                match Map::map_ref(mu, Fixnum::as_i64(cache_id) as usize, key) {
                    Some(_) => Symbol::keyword("t"),
                    None => Tag::nil(),
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_map_items(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let map = fp.argv[0];

        fp.value = match mu.fp_argv_check("mp-list", &[Type::Map], fp) {
            Ok(_) => Map::list(mu, map),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_map_size(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let map = fp.argv[0];

        fp.value = match mu.fp_argv_check("mp-size", &[Type::Map], fp) {
            Ok(_) => {
                let index_ref = block_on(mu.map_index.read());
                let cache_id = Map::cache_id(mu, map);

                match index_ref.get(&(Fixnum::as_i64(cache_id) as usize)) {
                    Some(hash) => Fixnum::as_tag(hash.len() as i64),
                    None => panic!(),
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn map() {
        assert_eq!(true, true)
    }
}
