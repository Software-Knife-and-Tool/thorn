//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu struct type
use crate::{
    core::{
        exception::{self, Condition, Exception},
        frame::Frame,
        indirect::IndirectTag,
        mu::{Core as _, Mu},
        types::{Tag, TagType, Type},
    },
    types::{
        cons::{Cons, ConsIter, Core as _},
        stream::{Core as _, Stream},
        symbol::{Core as _, Symbol},
        vecimage::{TypedVec, VecType, VectorIter},
        vector::Core as _,
    },
};

#[cfg(feature = "async")]
use futures::executor::block_on;

// a struct is a vector with an arbitrary type
pub struct Struct {
    pub stype: Tag,
    pub vector: Tag,
}

impl Struct {
    pub fn stype(mu: &Mu, tag: Tag) -> Tag {
        match Tag::type_of(mu, tag) {
            Type::Struct => {
                let struct_ = Self::to_image(mu, tag);

                struct_.stype
            }
            _ => panic!(),
        }
    }

    pub fn vector(mu: &Mu, tag: Tag) -> Tag {
        match Tag::type_of(mu, tag) {
            Type::Struct => {
                let struct_ = Self::to_image(mu, tag);

                struct_.vector
            }
            _ => panic!(),
        }
    }

    pub fn to_image(mu: &Mu, tag: Tag) -> Self {
        match Tag::type_of(mu, tag) {
            Type::Struct => match tag {
                Tag::Indirect(image) => {
                    #[cfg(feature = "async")]
                    let heap_ref = block_on(mu.heap.read());
                    #[cfg(feature = "no-async")]
                    let heap_ref = mu.heap.borrow();

                    Struct {
                        stype: Tag::from_slice(
                            heap_ref.of_length(image.offset() as usize, 8).unwrap(),
                        ),
                        vector: Tag::from_slice(
                            heap_ref.of_length(image.offset() as usize + 8, 8).unwrap(),
                        ),
                    }
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn to_tag(mu: &Mu, stype: Tag, vec: Vec<Tag>) -> Tag {
        match Tag::type_of(mu, stype) {
            Type::Keyword => {
                let vector = TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu);
                Struct { stype, vector }.evict(mu)
            }
            _ => panic!(),
        }
    }
}

// core
pub trait Core<'a> {
    fn new(_: &Mu, _: String, _: Vec<Tag>) -> Self;
    fn read(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn evict(&self, _: &Mu) -> Tag;
    fn view(_: &Mu, _: Tag) -> Tag;
}

impl<'a> Core<'a> for Struct {
    fn new(mu: &Mu, key: String, vec: Vec<Tag>) -> Self {
        Struct {
            stype: Symbol::keyword(&key),
            vector: TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu),
        }
    }

    fn view(mu: &Mu, tag: Tag) -> Tag {
        let image = Self::to_image(mu, tag);
        let vec = vec![image.stype, image.vector];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn write(mu: &Mu, tag: Tag, _: bool, stream: Tag) -> exception::Result<()> {
        match tag {
            Tag::Indirect(_) => {
                match mu.write_string("#s(".to_string(), stream) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }

                match mu.write(Self::to_image(mu, tag).stype, true, stream) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }

                for tag in VectorIter::new(mu, Self::to_image(mu, tag).vector) {
                    match mu.write_string(" ".to_string(), stream) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }

                    match mu.write(tag, false, stream) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                }

                mu.write_string(")".to_string(), stream)
            }
            _ => panic!(),
        }
    }

    fn read(mu: &Mu, stream: Tag) -> exception::Result<Tag> {
        match Stream::read_char(mu, stream) {
            Ok(Some(ch)) => match ch {
                '(' => {
                    let vec_list = match Cons::read(mu, stream) {
                        Ok(list) => {
                            if list.null_() {
                                return Err(Exception::new(Condition::Type, "read:st", Tag::nil()));
                            }
                            list
                        }
                        Err(_) => {
                            return Err(Exception::new(Condition::Syntax, "read:st", stream));
                        }
                    };

                    let stype = Cons::car(mu, vec_list);
                    match Tag::type_of(mu, stype) {
                        Type::Keyword => {
                            let mut vec = Vec::new();
                            for cons in ConsIter::new(mu, Cons::cdr(mu, vec_list)) {
                                vec.push(Cons::car(mu, cons));
                            }

                            Ok(Self::to_tag(mu, stype, vec))
                        }
                        _ => Err(Exception::new(Condition::Type, "read:st", stype)),
                    }
                }
                _ => Err(Exception::new(Condition::Eof, "read:st", stream)),
            },
            Ok(None) => Err(Exception::new(Condition::Eof, "read:st", stream)),
            Err(e) => Err(e),
        }
    }

    fn evict(&self, mu: &Mu) -> Tag {
        let image: &[[u8; 8]] = &[self.stype.as_slice(), self.vector.as_slice()];

        #[cfg(feature = "async")]
        let mut heap_ref = block_on(mu.heap.write());
        #[cfg(feature = "no-async")]
        let mut heap_ref = mu.heap.borrow_mut();

        Tag::Indirect(
            IndirectTag::new()
                .with_offset(heap_ref.alloc(image, Type::Struct as u8) as u64)
                .with_heap_id(1)
                .with_tag(TagType::Indirect),
        )
    }
}

// mu functions
pub trait MuFunction {
    fn mu_struct_type(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_struct_vector(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_make_struct(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Struct {
    fn mu_struct_type(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let tag = fp.argv[0];

        match Tag::type_of(mu, tag) {
            Type::Struct => {
                let image = Self::to_image(mu, tag);

                fp.value = image.stype;
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "st-type", tag)),
        }
    }

    fn mu_struct_vector(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let tag = fp.argv[0];

        match Tag::type_of(mu, tag) {
            Type::Struct => {
                let image = Self::to_image(mu, tag);

                fp.value = image.vector;
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "st-vec", tag)),
        }
    }

    fn mu_make_struct(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stype = fp.argv[0];
        let list = fp.argv[1];

        fp.value = match Tag::type_of(mu, stype) {
            Type::Keyword => {
                let mut vec = Vec::new();
                for cons in ConsIter::new(mu, list) {
                    vec.push(Cons::car(mu, cons));
                }

                let vector = TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu);

                Struct { stype, vector }.evict(mu)
            }
            _ => {
                return Err(Exception::new(Condition::Type, "make-st", stype));
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
