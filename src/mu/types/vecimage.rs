//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu image vector type
use crate::{
    core::{
        direct::{DirectInfo, DirectTag, DirectType},
        indirect::IndirectTag,
        mu::Mu,
        types::{Tag, TagType, Type},
    },
    types::{
        char::Char,
        fixnum::Fixnum,
        float::Float,
        symbol::{Core as _, Symbol},
        vector::{Core, Vector},
    },
};

#[cfg(feature = "async")]
use futures::executor::block_on;

pub struct VectorImage {
    pub vtype: Tag,  // type keyword
    pub length: Tag, // fixnum
}

pub enum IVec {
    Byte(Vec<u8>),
    Char(String),
    Fixnum(Vec<i64>),
    Float(Vec<f32>),
    T(Vec<Tag>),
}

// vector types
#[allow(dead_code)]
pub enum IndirectVector<'a> {
    Byte(&'a (VectorImage, IVec)),
    Char(&'a (VectorImage, IVec)),
    Fixnum(&'a (VectorImage, IVec)),
    Float(&'a (VectorImage, IVec)),
    T(&'a (VectorImage, IVec)),
}

pub trait IVector {
    const IMAGE_NBYTES: usize = 2 * 8; // bytes in image
    fn image(_: &VectorImage) -> Vec<[u8; 8]>;
    fn evict(&self, _: &Mu) -> Tag;
    fn ref_(_: &Mu, _: Tag, _: usize) -> Option<Tag>;
}

impl<'a> IVector for IndirectVector<'a> {
    fn image(image: &VectorImage) -> Vec<[u8; 8]> {
        let slices = vec![image.vtype.as_slice(), image.length.as_slice()];

        slices
    }

    fn evict(&self, mu: &Mu) -> Tag {
        match self {
            IndirectVector::Byte((image, ivec)) => {
                let slices = Self::image(image);

                let data = match ivec {
                    IVec::Byte(vec_u8) => &vec_u8[..],
                    _ => panic!(),
                };

                #[cfg(feature = "async")]
                let mut heap_ref = block_on(mu.heap.write());
                #[cfg(not(feature = "async"))]
                let mut heap_ref = mu.heap.borrow_mut();

                Tag::Indirect(
                    IndirectTag::new()
                        .with_offset(heap_ref.valloc(&slices, data, Type::Vector as u8) as u64)
                        .with_heap_id(1)
                        .with_tag(TagType::Vector),
                )
            }
            IndirectVector::Char((image, ivec)) => {
                let slices = Self::image(image);

                let data = match ivec {
                    IVec::Char(string) => string.as_bytes(),
                    _ => panic!(),
                };

                #[cfg(feature = "async")]
                let mut heap_ref = block_on(mu.heap.write());
                #[cfg(not(feature = "async"))]
                let mut heap_ref = mu.heap.borrow_mut();

                Tag::Indirect(
                    IndirectTag::new()
                        .with_offset(heap_ref.valloc(&slices, data, Type::Vector as u8) as u64)
                        .with_heap_id(1)
                        .with_tag(TagType::Vector),
                )
            }
            IndirectVector::T((image, vec)) => {
                let mut slices = Self::image(image);

                match vec {
                    IVec::T(vec) => {
                        for tag in vec {
                            slices.push(tag.as_slice());
                        }
                    }
                    _ => panic!(),
                }

                #[cfg(feature = "async")]
                let mut heap_ref = block_on(mu.heap.write());
                #[cfg(not(feature = "async"))]
                let mut heap_ref = mu.heap.borrow_mut();

                Tag::Indirect(
                    IndirectTag::new()
                        .with_offset(heap_ref.alloc(&slices, Type::Vector as u8) as u64)
                        .with_heap_id(1)
                        .with_tag(TagType::Vector),
                )
            }
            IndirectVector::Fixnum((image, vec)) => {
                let mut slices = Self::image(image);

                match vec {
                    IVec::Fixnum(vec) => {
                        for n in vec {
                            slices.push(n.to_le_bytes());
                        }
                    }
                    _ => panic!(),
                }

                #[cfg(feature = "async")]
                let mut heap_ref = block_on(mu.heap.write());
                #[cfg(not(feature = "async"))]
                let mut heap_ref = mu.heap.borrow_mut();

                Tag::Indirect(
                    IndirectTag::new()
                        .with_offset(heap_ref.alloc(&slices, Type::Vector as u8) as u64)
                        .with_heap_id(1)
                        .with_tag(TagType::Vector),
                )
            }
            IndirectVector::Float((image, vec)) => {
                let data = match vec {
                    IVec::Float(vec_u4) => {
                        let mut vec_u8 = Vec::<u8>::new();
                        for float in vec_u4 {
                            let slice = float.to_le_bytes();
                            vec_u8.push(slice[0]);
                            vec_u8.push(slice[1]);
                            vec_u8.push(slice[2]);
                            vec_u8.push(slice[3]);
                        }
                        vec_u8
                    }
                    _ => panic!(),
                };

                #[cfg(feature = "async")]
                let mut heap_ref = block_on(mu.heap.write());
                #[cfg(not(feature = "async"))]
                let mut heap_ref = mu.heap.borrow_mut();

                Tag::Indirect(
                    IndirectTag::new()
                        .with_offset(
                            heap_ref.valloc(&Self::image(image), &data, Type::Vector as u8) as u64,
                        )
                        .with_heap_id(1)
                        .with_tag(TagType::Vector),
                )
            }
        }
    }

    fn ref_(mu: &Mu, vector: Tag, index: usize) -> Option<Tag> {
        let image = Vector::to_image(mu, vector);

        let len = Fixnum::as_i64(mu, image.length) as usize;
        if index >= len {
            return None;
        }

        match Vector::to_type(image.vtype).unwrap() {
            Type::Byte => match vector {
                Tag::Indirect(image) => {
                    #[cfg(feature = "async")]
                    let heap_ref = block_on(mu.heap.read());
                    #[cfg(not(feature = "async"))]
                    let heap_ref = mu.heap.borrow();

                    let slice = heap_ref
                        .of_length(image.offset() as usize + Self::IMAGE_NBYTES + index, 1)
                        .unwrap();

                    Some(Fixnum::as_tag(slice[0] as i64))
                }
                _ => panic!(),
            },
            Type::Char => match vector {
                Tag::Indirect(image) => {
                    #[cfg(feature = "async")]
                    let heap_ref = block_on(mu.heap.read());
                    #[cfg(not(feature = "async"))]
                    let heap_ref = mu.heap.borrow();

                    let slice = heap_ref
                        .of_length(image.offset() as usize + Self::IMAGE_NBYTES + index, 1)
                        .unwrap();

                    Some(Char::as_tag(slice[0] as char))
                }
                _ => panic!(),
            },
            Type::T => match vector {
                Tag::Indirect(image) => {
                    #[cfg(feature = "async")]
                    let heap_ref = block_on(mu.heap.read());
                    #[cfg(not(feature = "async"))]
                    let heap_ref = mu.heap.borrow();

                    Some(Tag::from_slice(
                        heap_ref
                            .of_length(
                                image.offset() as usize + Self::IMAGE_NBYTES + (index * 8),
                                8,
                            )
                            .unwrap(),
                    ))
                }
                _ => panic!(),
            },
            Type::Fixnum => match vector {
                Tag::Indirect(image) => {
                    #[cfg(feature = "async")]
                    let heap_ref = block_on(mu.heap.read());
                    #[cfg(not(feature = "async"))]
                    let heap_ref = mu.heap.borrow();

                    let slice = heap_ref
                        .of_length(
                            image.offset() as usize + Self::IMAGE_NBYTES + (index * 8),
                            8,
                        )
                        .unwrap();

                    Some(Fixnum::as_tag(i64::from_le_bytes(
                        slice[0..8].try_into().unwrap(),
                    )))
                }
                _ => panic!(),
            },
            Type::Float => match vector {
                Tag::Indirect(image) => {
                    #[cfg(feature = "async")]
                    let heap_ref = block_on(mu.heap.read());
                    #[cfg(not(feature = "async"))]
                    let heap_ref = mu.heap.borrow();

                    let slice = heap_ref
                        .of_length(
                            image.offset() as usize + Self::IMAGE_NBYTES + (index * 4),
                            4,
                        )
                        .unwrap();

                    Some(Float::as_tag(f32::from_le_bytes(
                        slice[0..4].try_into().unwrap(),
                    )))
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
}

/// typed vector allocation
pub struct TypedVec<T: VecType> {
    pub vec: T,
}

pub trait VecType {
    fn to_vector(&self) -> Vector;
}

impl VecType for String {
    fn to_vector(&self) -> Vector {
        let len = self.len();

        if len > DirectTag::DIRECT_STR_MAX {
            let image = VectorImage {
                vtype: Symbol::keyword("char"),
                length: Fixnum::as_tag(self.len() as i64),
            };

            Vector::Indirect((image, IVec::Char(self.to_string())))
        } else {
            let mut data: [u8; 8] = 0u64.to_le_bytes();

            for (src, dst) in self.as_bytes().iter().zip(data.iter_mut()) {
                *dst = *src
            }

            Vector::Direct(DirectTag::to_direct(
                u64::from_le_bytes(data),
                DirectInfo::Length(len),
                DirectType::Byte,
            ))
        }
    }
}

impl VecType for Vec<Tag> {
    fn to_vector(&self) -> Vector {
        let image = VectorImage {
            vtype: Symbol::keyword("t"),
            length: Fixnum::as_tag(self.len() as i64),
        };

        Vector::Indirect((image, IVec::T(self.to_vec())))
    }
}

impl VecType for Vec<i64> {
    fn to_vector(&self) -> Vector {
        let image = VectorImage {
            vtype: Symbol::keyword("fixnum"),
            length: Fixnum::as_tag(self.len() as i64),
        };

        Vector::Indirect((image, IVec::Fixnum(self.to_vec())))
    }
}

impl VecType for Vec<u8> {
    fn to_vector(&self) -> Vector {
        let image = VectorImage {
            vtype: Symbol::keyword("byte"),
            length: Fixnum::as_tag(self.len() as i64),
        };

        Vector::Indirect((image, IVec::Byte(self.to_vec())))
    }
}

impl VecType for Vec<f32> {
    fn to_vector(&self) -> Vector {
        let image = VectorImage {
            vtype: Symbol::keyword("float"),
            length: Fixnum::as_tag(self.len() as i64),
        };

        Vector::Indirect((image, IVec::Float(self.to_vec())))
    }
}

/// iterator
pub struct VectorIter<'a> {
    mu: &'a Mu,
    pub vec: Tag,
    pub index: usize,
}

impl<'a> VectorIter<'a> {
    pub fn new(mu: &'a Mu, vec: Tag) -> Self {
        Self { mu, vec, index: 0 }
    }
}

impl<'a> Iterator for VectorIter<'a> {
    type Item = Tag;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= Vector::length(self.mu, self.vec) {
            None
        } else {
            let el = Vector::ref_(self.mu, self.vec, self.index);
            self.index += 1;

            Some(el.unwrap())
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
