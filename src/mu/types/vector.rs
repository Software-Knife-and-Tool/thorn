//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu vector type
use {
    crate::{
        core::{
            direct::{DirectInfo, DirectTag, DirectType},
            exception::{self, Condition, Exception},
            frame::Frame,
            funcall::Core as _,
            heap::{Core as _, Heap},
            mu::{Core as _, Mu},
            readtable::{map_char_syntax, SyntaxType},
            stream,
            types::{Tag, Type},
        },
        types::{
            char::Char,
            cons::{Cons, ConsIter, Core as _},
            fixnum::Fixnum,
            float::Float,
            stream::{Core as _, Stream},
            symbol::{Core as _, Symbol},
            vecimage::{IVec, IVector, IndirectVector, VectorImage},
            vecimage::{TypedVec, VecType, VectorIter},
        },
    },
    std::str,
};

use futures::executor::block_on;

pub enum Vector {
    Direct(Tag),
    Indirect((VectorImage, IVec)),
}

lazy_static! {
    static ref VTYPEMAP: Vec<(Tag, Type)> = vec![
        (Symbol::keyword("t"), Type::T),
        (Symbol::keyword("char"), Type::Char),
        (Symbol::keyword("byte"), Type::Byte),
        (Symbol::keyword("fixnum"), Type::Fixnum),
        (Symbol::keyword("float"), Type::Float),
    ];
}

impl Vector {
    pub fn to_type(keyword: Tag) -> Option<Type> {
        VTYPEMAP
            .iter()
            .copied()
            .find(|tab| keyword.eq_(&tab.0))
            .map(|tab| tab.1)
    }

    pub fn to_image(mu: &Mu, tag: Tag) -> VectorImage {
        match tag.type_of() {
            Type::Vector => match tag {
                Tag::Indirect(image) => {
                    let heap_ref = block_on(mu.heap.read());

                    VectorImage {
                        vtype: Tag::from_slice(
                            heap_ref.image_slice(image.image_id() as usize, 8).unwrap(),
                        ),
                        length: Tag::from_slice(
                            heap_ref
                                .image_slice(image.image_id() as usize + 8, 8)
                                .unwrap(),
                        ),
                    }
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn type_of(mu: &Mu, vector: Tag) -> Type {
        match vector {
            Tag::Direct(_) => Type::Char,
            Tag::Indirect(_) => {
                let image = Self::to_image(mu, vector);

                match VTYPEMAP
                    .iter()
                    .copied()
                    .find(|desc| image.vtype.eq_(&desc.0))
                {
                    Some(desc) => desc.1,
                    None => panic!(),
                }
            }
        }
    }

    pub fn length(mu: &Mu, vector: Tag) -> usize {
        match vector {
            Tag::Direct(direct) => direct.info() as usize,
            Tag::Indirect(_) => {
                let image = Self::to_image(mu, vector);
                Fixnum::as_i64(image.length) as usize
            }
        }
    }
}

// core
pub trait Core<'a> {
    fn as_string(_: &Mu, _: Tag) -> String;
    fn evict(&self, _: &Mu) -> Tag;
    fn from_string(_: &str) -> Vector;
    fn gc_mark(_: &Mu, _: Tag);
    fn heap_size(_: &Mu, _: Tag) -> usize;
    fn read(_: &Mu, _: char, _: Tag) -> exception::Result<Tag>;
    fn r#ref(_: &Mu, _: Tag, _: usize) -> Option<Tag>;
    fn view(_: &Mu, _: Tag) -> Tag;
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
}

impl<'a> Core<'a> for Vector {
    fn view(mu: &Mu, vector: Tag) -> Tag {
        let vec = vec![
            Fixnum::as_tag(Self::length(mu, vector) as i64),
            match Tag::type_key(Self::type_of(mu, vector)) {
                Some(key) => key,
                None => panic!(),
            },
        ];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn heap_size(mu: &Mu, vector: Tag) -> usize {
        match vector {
            Tag::Direct(_) => std::mem::size_of::<DirectTag>(),
            Tag::Indirect(_) => {
                let len = Self::length(mu, vector);
                let size = match Vector::type_of(mu, vector) {
                    Type::Byte | Type::Char => 1,
                    Type::Fixnum | Type::Float | Type::T => 8,
                    _ => panic!(),
                };

                std::mem::size_of::<VectorImage>() + (size * len)
            }
        }
    }

    fn from_string(str: &str) -> Vector {
        let len = str.len();

        if len > DirectTag::DIRECT_STR_MAX {
            TypedVec::<String> {
                vec: str.to_string(),
            }
            .vec
            .to_vector()
        } else {
            let mut data: [u8; 8] = 0u64.to_le_bytes();

            for (src, dst) in str.as_bytes().iter().zip(data.iter_mut()) {
                *dst = *src
            }

            Vector::Direct(DirectTag::to_direct(
                u64::from_le_bytes(data),
                DirectInfo::Length(len),
                DirectType::Byte,
            ))
        }
    }

    fn as_string(mu: &Mu, tag: Tag) -> String {
        match tag.type_of() {
            Type::Vector => match tag {
                Tag::Direct(dir) => match dir.dtype() {
                    DirectType::Byte => str::from_utf8(&dir.data().to_le_bytes()).unwrap()
                        [..dir.info() as usize]
                        .to_string(),
                    _ => panic!(),
                },
                Tag::Indirect(image) => {
                    let heap_ref = block_on(mu.heap.read());
                    let vec: VectorImage = Self::to_image(mu, tag);

                    str::from_utf8(
                        heap_ref
                            .image_slice(
                                (image.image_id() + 16) as usize,
                                Fixnum::as_i64(vec.length) as usize,
                            )
                            .unwrap(),
                    )
                    .unwrap()
                    .to_string()
                }
            },
            _ => panic!(),
        }
    }

    fn gc_mark(mu: &Mu, vector: Tag) {
        match vector {
            Tag::Direct(_) => (),
            Tag::Indirect(_) => {
                let mark = Heap::mark(mu, vector).unwrap();
                let vtype = Self::type_of(mu, vector);
                let length = Self::length(mu, vector);

                if !mark && vtype == Type::T {
                    for index in 0..length {
                        mu.gc_mark(Self::r#ref(mu, vector, index).unwrap())
                    }
                }
            }
        }
    }

    fn write(mu: &Mu, vector: Tag, escape: bool, stream: Tag) -> exception::Result<()> {
        match vector {
            Tag::Direct(_) => match str::from_utf8(&vector.data(mu).to_le_bytes()) {
                Ok(s) => {
                    if escape {
                        <Mu as stream::Core>::write_string(mu, "\"", stream).unwrap()
                    }

                    for nth in 0..DirectTag::length(vector) {
                        match Stream::write_char(mu, stream, s.as_bytes()[nth] as char) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                    }

                    if escape {
                        <Mu as stream::Core>::write_string(mu, "\"", stream).unwrap()
                    }

                    Ok(())
                }
                Err(_) => panic!(),
            },
            Tag::Indirect(_) => match Self::type_of(mu, vector) {
                Type::Char => {
                    if escape {
                        match <Mu as stream::Core>::write_string(mu, "\"", stream) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                    }

                    for ch in VectorIter::new(mu, vector) {
                        match <Mu as stream::Core>::write(mu, ch, false, stream) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                    }

                    if escape {
                        match <Mu as stream::Core>::write_string(mu, "\"", stream) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                    }

                    Ok(())
                }
                _ => {
                    match <Mu as stream::Core>::write_string(mu, "#(", stream) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                    match <Mu as stream::Core>::write(
                        mu,
                        Self::to_image(mu, vector).vtype,
                        true,
                        stream,
                    ) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }

                    for tag in VectorIter::new(mu, vector) {
                        match <Mu as stream::Core>::write_string(mu, " ", stream) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }

                        match <Mu as stream::Core>::write(mu, tag, false, stream) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                    }

                    <Mu as stream::Core>::write_string(mu, ")", stream)
                }
            },
        }
    }

    fn read(mu: &Mu, syntax: char, stream: Tag) -> exception::Result<Tag> {
        match syntax {
            '"' => {
                let mut str: String = String::new();

                loop {
                    match Stream::read_char(mu, stream) {
                        Ok(Some('"')) => break,
                        Ok(Some(ch)) => match map_char_syntax(ch).unwrap() {
                            SyntaxType::Escape => match Stream::read_char(mu, stream) {
                                Ok(Some(ch)) => str.push(ch),
                                Ok(None) => {
                                    return Err(Exception::new(Condition::Eof, "read:sv", stream));
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            },
                            _ => str.push(ch),
                        },
                        Ok(None) => {
                            return Err(Exception::new(Condition::Eof, "read:sv", stream));
                        }
                        Err(e) => return Err(e),
                    }
                }

                Ok(Self::from_string(&str).evict(mu))
            }
            '(' => {
                let vec_list = match Cons::read(mu, stream) {
                    Ok(list) => {
                        if list.null_() {
                            return Err(Exception::new(Condition::Type, "read:sv", Tag::nil()));
                        }
                        list
                    }
                    Err(_) => {
                        return Err(Exception::new(Condition::Syntax, "read:sv", stream));
                    }
                };

                let vec_type = Cons::car(mu, vec_list);

                match VTYPEMAP.iter().copied().find(|tab| vec_type.eq_(&tab.0)) {
                    Some(tab) => match tab.1 {
                        Type::T => {
                            let mut vec = Vec::new();
                            for cons in ConsIter::new(mu, Cons::cdr(mu, vec_list)) {
                                vec.push(Cons::car(mu, cons));
                            }
                            Ok(TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu))
                        }
                        Type::Char => {
                            let mut vec = String::new();
                            for cons in ConsIter::new(mu, Cons::cdr(mu, vec_list)) {
                                let el = Cons::car(mu, cons);
                                match el.type_of() {
                                    Type::Char => {
                                        let ch = Char::as_char(mu, el);
                                        vec.push(ch)
                                    }
                                    _ => {
                                        return Err(Exception::new(Condition::Type, "read:sv", el))
                                    }
                                }
                            }

                            Ok(TypedVec::<String> { vec }.vec.to_vector().evict(mu))
                        }
                        Type::Byte => {
                            let mut vec = Vec::<u8>::new();
                            for cons in ConsIter::new(mu, Cons::cdr(mu, vec_list)) {
                                let el = Cons::car(mu, cons);
                                match el.type_of() {
                                    Type::Fixnum => {
                                        let byte = Fixnum::as_i64(el);
                                        if !(0..255).contains(&byte) {
                                            return Err(Exception::new(
                                                Condition::Range,
                                                "read:sv",
                                                el,
                                            ));
                                        }
                                        vec.push(byte as u8)
                                    }
                                    _ => {
                                        return Err(Exception::new(Condition::Type, "read:sv", el))
                                    }
                                }
                            }

                            Ok(TypedVec::<Vec<u8>> { vec }.vec.to_vector().evict(mu))
                        }
                        Type::Fixnum => {
                            let mut vec = Vec::new();

                            for cons in ConsIter::new(mu, Cons::cdr(mu, vec_list)) {
                                let el = Cons::car(mu, cons);
                                match el.type_of() {
                                    Type::Fixnum => vec.push(Fixnum::as_i64(el)),
                                    _ => {
                                        return Err(Exception::new(Condition::Type, "read:sv", el))
                                    }
                                }
                            }

                            Ok(TypedVec::<Vec<i64>> { vec }.vec.to_vector().evict(mu))
                        }
                        Type::Float => {
                            let mut vec = Vec::new();

                            for cons in ConsIter::new(mu, Cons::cdr(mu, vec_list)) {
                                let el = Cons::car(mu, cons);
                                match el.type_of() {
                                    Type::Float => vec.push(Float::as_f32(mu, el)),
                                    _ => {
                                        return Err(Exception::new(Condition::Type, "read:sv", el))
                                    }
                                }
                            }

                            Ok(TypedVec::<Vec<f32>> { vec }.vec.to_vector().evict(mu))
                        }
                        _ => panic!(),
                    },
                    None => Err(Exception::new(Condition::Type, "read:sv", vec_type)),
                }
            }
            _ => panic!(),
        }
    }

    fn evict(&self, mu: &Mu) -> Tag {
        match self {
            Vector::Direct(tag) => *tag,
            Vector::Indirect(desc) => {
                let (_, ivec) = desc;
                match ivec {
                    IVec::T(_) => IndirectVector::T(desc).evict(mu),
                    IVec::Char(_) => IndirectVector::Char(desc).evict(mu),
                    IVec::Byte(_) => IndirectVector::Byte(desc).evict(mu),
                    IVec::Fixnum(_) => IndirectVector::Fixnum(desc).evict(mu),
                    IVec::Float(_) => IndirectVector::Float(desc).evict(mu),
                }
            }
        }
    }

    fn r#ref(mu: &Mu, vector: Tag, index: usize) -> Option<Tag> {
        match vector.type_of() {
            Type::Vector => match vector {
                Tag::Direct(_direct) => {
                    Some(Char::as_tag(vector.data(mu).to_le_bytes()[index] as char))
                }
                Tag::Indirect(_) => IndirectVector::r#ref(mu, vector, index),
            },
            _ => panic!(),
        }
    }
}

/// mu functions
pub trait MuFunction {
    fn mu_type(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_length(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_make_vector(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_svref(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Vector {
    fn mu_make_vector(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let type_sym = fp.argv[0];
        let list = fp.argv[1];

        fp.value = match Self::to_type(type_sym) {
            Some(vtype) => match vtype {
                Type::Null => return Err(Exception::new(Condition::Type, "make-sv", type_sym)),
                Type::T => {
                    let mut vec = Vec::new();
                    for cons in ConsIter::new(mu, list) {
                        vec.push(Cons::car(mu, cons));
                    }

                    TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
                }
                Type::Char => {
                    let mut vec = String::new();

                    for cons in ConsIter::new(mu, list) {
                        let el = Cons::car(mu, cons);

                        match el.type_of() {
                            Type::Char => {
                                vec.push(Char::as_char(mu, el));
                            }
                            _ => return Err(Exception::new(Condition::Type, "make-sv", el)),
                        }
                    }

                    TypedVec::<String> { vec }.vec.to_vector().evict(mu)
                }
                Type::Byte => {
                    let mut vec = Vec::<u8>::new();

                    for cons in ConsIter::new(mu, list) {
                        let el = Cons::car(mu, cons);

                        match el.type_of() {
                            Type::Fixnum => {
                                let byte = Fixnum::as_i64(el);

                                if !(0..=255).contains(&byte) {
                                    return Err(Exception::new(Condition::Range, "make-sv", el));
                                }

                                vec.push(byte as u8);
                            }
                            _ => return Err(Exception::new(Condition::Type, "make-sv", el)),
                        }
                    }

                    TypedVec::<Vec<u8>> { vec }.vec.to_vector().evict(mu)
                }
                Type::Fixnum => {
                    let mut vec = Vec::new();
                    for cons in ConsIter::new(mu, list) {
                        let el = Cons::car(mu, cons);

                        match el.type_of() {
                            Type::Fixnum => {
                                vec.push(Fixnum::as_i64(el));
                            }
                            _ => return Err(Exception::new(Condition::Type, "make-sv", el)),
                        }
                    }

                    TypedVec::<Vec<i64>> { vec }.vec.to_vector().evict(mu)
                }
                Type::Float => {
                    let mut vec = Vec::new();
                    for cons in ConsIter::new(mu, list) {
                        let el = Cons::car(mu, cons);

                        match el.type_of() {
                            Type::Float => {
                                vec.push(Float::as_f32(mu, el));
                            }
                            _ => return Err(Exception::new(Condition::Type, "make-sv", el)),
                        }
                    }

                    TypedVec::<Vec<f32>> { vec }.vec.to_vector().evict(mu)
                }
                _ => {
                    return Err(Exception::new(Condition::Type, "make-sv", type_sym));
                }
            },
            None => {
                return Err(Exception::new(Condition::Type, "make-sv", type_sym));
            }
        };

        Ok(())
    }

    fn mu_svref(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let vector = fp.argv[0];
        let index = fp.argv[1];

        fp.value = match mu.fp_argv_check("sv-ref", &[Type::Vector, Type::Fixnum], fp) {
            Ok(_) => {
                let nth = Fixnum::as_i64(index);

                if nth < 0 || nth as usize >= Self::length(mu, vector) {
                    return Err(Exception::new(Condition::Range, "sv-ref", index));
                }

                match Self::r#ref(mu, vector, nth as usize) {
                    Some(nth) => nth,
                    None => panic!(),
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_type(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let vector = fp.argv[0];

        fp.value = match mu.fp_argv_check("sv-type", &[Type::Vector], fp) {
            Ok(_) => match Tag::type_key(Vector::type_of(mu, vector)) {
                Some(key) => key,
                None => panic!(),
            },
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_length(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let vector = fp.argv[0];

        fp.value = match mu.fp_argv_check("sv-len", &[Type::Vector], fp) {
            Ok(_) => Fixnum::as_tag(Self::length(mu, vector) as i64),
            Err(e) => return Err(e),
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
