//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu maps
use {
    crate::{
        core::{
            exception::{self, Condition, Exception},
            frame::Frame,
            heap::Core as _,
            indirect::IndirectTag,
            mu::Mu,
            stream,
            types::{Tag, TagType, Type},
        },
        types::{
            cons::{Cons, Core as _},
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
    fn new(mu: &Mu) -> Self {
        let mut index_ref = block_on(mu.map_index.write());
        let cache_id = index_ref.len();

        index_ref.insert(cache_id, HashMap::<u64, Tag>::new());

        Map {
            cache_id: Fixnum::as_tag(cache_id as i64),
            list: Tag::nil(),
        }
    }

    fn to_image(mu: &Mu, tag: Tag) -> Self {
        match Tag::type_of(tag) {
            Type::Map => match tag {
                Tag::Indirect(main) => {
                    let heap_ref = block_on(mu.heap.read());

                    Map {
                        cache_id: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize, 8).unwrap(),
                        ),
                        list: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize + 8, 8).unwrap(),
                        ),
                    }
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    fn cache_id(mu: &Mu, map: Tag) -> Tag {
        match Tag::type_of(map) {
            Type::Map => match map {
                Tag::Indirect(_) => Self::to_image(mu, map).cache_id,
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    fn list(mu: &Mu, map: Tag) -> Tag {
        match Tag::type_of(map) {
            Type::Map => match map {
                Tag::Indirect(_) => Self::to_image(mu, map).list,
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    // do we need a lock at all?
    fn map_get(mu: &Mu, cache_id: usize, key: Tag) -> Option<Tag> {
        let index_ref = block_on(mu.map_index.read());

        match index_ref.get(&cache_id) {
            Some(hash) => hash.get(&key.as_u64()).copied(),
            None => None,
        }
    }

    fn map_add(mu: &Mu, cache_id: usize, key: Tag, value: Tag) -> Option<()> {
        let mut index_ref = block_on(mu.map_index.write());

        match index_ref.get_mut(&cache_id) {
            Some(hash) => {
                if let std::collections::hash_map::Entry::Vacant(e) = hash.entry(key.as_u64()) {
                    e.insert(value);
                    Some(())
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn evict(&self, mu: &Mu) -> Tag {
        let image: &[[u8; 8]] = &[self.cache_id.as_slice(), self.list.as_slice()];

        let mut heap_ref = block_on(mu.heap.write());
        let ind = IndirectTag::new()
            .with_offset(heap_ref.alloc(image, Type::Map as u8) as u64)
            .with_heap_id(1)
            .with_tag(TagType::Map);

        Tag::Indirect(ind)
    }

    fn update(mu: &Mu, image: &Map, tag: Tag) {
        let slices: &[[u8; 8]] = &[image.cache_id.as_slice(), image.list.as_slice()];

        let offset = match tag {
            Tag::Indirect(heap) => heap.offset(),
            _ => panic!(),
        } as usize;

        let mut heap_ref = block_on(mu.heap.write());

        heap_ref.write_image(slices, offset);
    }
}

pub trait Core {
    fn evict(&self, _: &Mu) -> Tag;
    fn gc_mark(_: &Mu, _: Tag);
    fn size_of(_: &Mu, _: Tag) -> usize;
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn view(_: &Mu, _: Tag) -> Tag;
}

impl Core for Map {
    fn gc_mark(mu: &Mu, tag: Tag) {
        match tag {
            Tag::Direct(_) => {
                // GcMark(env, car(ptr));
                // GcMark(env, cdr(ptr));
            }
            Tag::Indirect(indir) => {
                let heap_ref = block_on(mu.heap.read());
                let mark = heap_ref.image_refbit(indir.offset() as usize).unwrap();

                if !mark {
                    // GcMark(env, ptr)
                    // GcMark(env, car(ptr));
                    // GcMark(env, cdr(ptr));
                }
            }
        }
    }

    fn view(mu: &Mu, map: Tag) -> Tag {
        let vec = vec![Self::cache_id(mu, map), Self::list(mu, map)];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn size_of(mu: &Mu, symbol: Tag) -> usize {
        std::mem::size_of::<Map>() + Mu::size_of(mu, Self::list(mu, symbol)).unwrap()
    }

    fn write(mu: &Mu, map: Tag, _: bool, stream: Tag) -> exception::Result<()> {
        match Tag::type_of(map) {
            Type::Map => {
                let cache_id = Fixnum::as_i64(Self::cache_id(mu, map));
                let index_ref = block_on(mu.map_index.write());
                let size = index_ref.len();

                <Mu as stream::Core>::write_string(
                    mu,
                    format!("#<:map [size:{size}, tag:{}]>", cache_id),
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
            .with_offset(heap_ref.alloc(image, Type::Map as u8) as u64)
            .with_heap_id(1)
            .with_tag(TagType::Function);

        Tag::Indirect(ind)
    }
}

pub trait MuFunction {
    fn mu_make_map(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_map_add(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_map_get(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_map_has(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_map_list(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_map_size(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_make_map(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = Map::new(mu).evict(mu);
        Ok(())
    }

    fn mu_map_add(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let map = fp.argv[0];
        let key = fp.argv[1];
        let value = fp.argv[2];

        fp.value = Cons::new(key, value).evict(mu);

        match Tag::type_of(map) {
            Type::Map => {
                let cache_id = Map::cache_id(mu, map);

                match Map::map_add(mu, Fixnum::as_i64(cache_id) as usize, key, value) {
                    Some(_) => {
                        let mut image = Map::to_image(mu, map);

                        image.list = Cons::new(fp.value, image.list).evict(mu);
                        Map::update(mu, &image, map);
                    }
                    None => return Err(Exception::new(Condition::Range, "mp-add", map)),
                }
            }
            _ => return Err(Exception::new(Condition::Type, "mp-add", map)),
        }

        Ok(())
    }

    fn mu_map_get(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let map = fp.argv[0];
        let key = fp.argv[1];

        fp.value = match Tag::type_of(map) {
            Type::Map => {
                let cache_id = Map::cache_id(mu, map);

                match Map::map_get(mu, Fixnum::as_i64(cache_id) as usize, key) {
                    Some(value) => value,
                    None => return Err(Exception::new(Condition::Range, "map-get", key)),
                }
            }
            _ => return Err(Exception::new(Condition::Type, "map-get", map)),
        };

        Ok(())
    }

    fn mu_map_has(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let map = fp.argv[0];
        let key = fp.argv[1];

        fp.value = match Tag::type_of(map) {
            Type::Map => {
                let cache_id = Map::cache_id(mu, map);

                match Map::map_get(mu, Fixnum::as_i64(cache_id) as usize, key) {
                    Some(_) => Symbol::keyword("t"),
                    None => Tag::nil(),
                }
            }
            _ => return Err(Exception::new(Condition::Type, "map-has", map)),
        };

        Ok(())
    }

    fn mu_map_list(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let map = fp.argv[0];

        fp.value = match Tag::type_of(map) {
            Type::Map => Map::list(mu, map),
            _ => return Err(Exception::new(Condition::Type, "mp-list", map)),
        };

        Ok(())
    }

    fn mu_map_size(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let map = fp.argv[0];

        fp.value = match Tag::type_of(map) {
            Type::Map => {
                let index_ref = block_on(mu.map_index.read());
                let cache_id = Map::cache_id(mu, map);

                match index_ref.get(&(Fixnum::as_i64(cache_id) as usize)) {
                    Some(hash) => Fixnum::as_tag(hash.len() as i64),
                    None => panic!(),
                }
            }
            _ => return Err(Exception::new(Condition::Type, "map-size", map)),
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
