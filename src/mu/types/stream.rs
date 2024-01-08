//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu stream type
#![allow(unused_braces)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::identity_op)]

use crate::{
    core::{
        exception::{self, Condition, Exception},
        heap::{Core as _, Heap},
        indirect::IndirectTag,
        mu::Mu,
        stream,
        types::{Tag, TagType, Type},
    },
    system::{stream::Core as _, sys::System},
    types::{
        char::Char,
        fixnum::Fixnum,
        symbol::{Core as _, Symbol},
        vecimage::{TypedVec, VecType},
        vector::Core as _,
    },
};

use futures::executor::block_on;

// stream struct
pub struct Stream {
    pub stream_id: Tag, // system stream id (fixnum)
    pub direction: Tag, // :input | :output | :bidir (keyword)
    pub eof: Tag,       // end of file flag (bool)
    pub unch: Tag,      // pushbask for input streams (() | character tag)
}

impl Stream {
    pub fn evict(&self, mu: &Mu) -> Tag {
        let slices: &[[u8; 8]] = &[
            self.stream_id.as_slice(),
            self.direction.as_slice(),
            self.eof.as_slice(),
            self.unch.as_slice(),
        ];

        let mut heap_ref = block_on(mu.heap.write());

        Tag::Indirect(
            IndirectTag::new()
                .with_image_id(heap_ref.alloc(slices, Type::Stream as u8) as u64)
                .with_heap_id(1)
                .with_tag(TagType::Stream),
        )
    }

    pub fn to_image(mu: &Mu, tag: Tag) -> Stream {
        match tag.type_of() {
            Type::Stream => match tag {
                Tag::Indirect(main) => {
                    let heap_ref = block_on(mu.heap.read());
                    let image = Stream {
                        stream_id: Tag::from_slice(
                            heap_ref.image_slice(main.image_id() as usize, 8).unwrap(),
                        ),
                        direction: Tag::from_slice(
                            heap_ref
                                .image_slice(main.image_id() as usize + 8, 8)
                                .unwrap(),
                        ),
                        eof: Tag::from_slice(
                            heap_ref
                                .image_slice(main.image_id() as usize + 16, 8)
                                .unwrap(),
                        ),
                        unch: Tag::from_slice(
                            heap_ref
                                .image_slice(main.image_id() as usize + 24, 8)
                                .unwrap(),
                        ),
                    };

                    image
                }
                _ => panic!(),
            },
            _ => panic!("stream type botch {:?}", tag.type_of()),
        }
    }

    pub fn update(mu: &Mu, image: &Stream, stream: Tag) {
        let slices: &[[u8; 8]] = &[
            image.stream_id.as_slice(),
            image.direction.as_slice(),
            image.eof.as_slice(),
            image.unch.as_slice(),
        ];

        let offset = match stream {
            Tag::Indirect(heap) => heap.image_id(),
            _ => panic!(),
        } as usize;

        let mut heap_ref = block_on(mu.heap.write());

        heap_ref.write_image(slices, offset);
    }
}

pub trait Core {
    fn close(_: &Mu, _: Tag);
    fn gc_mark(_: &Mu, _: Tag);
    fn get_string(_: &Mu, _: Tag) -> exception::Result<String>;
    fn heap_size(_: &Mu, _: Tag) -> usize;
    fn is_eof(_: &Mu, _: Tag) -> bool;
    fn clear_eof(mu: &Mu, stream: Tag);
    fn is_open(_: &Mu, _: Tag) -> bool;
    fn read_byte(_: &Mu, _: Tag) -> exception::Result<Option<u8>>;
    fn read_char(_: &Mu, _: Tag) -> exception::Result<Option<char>>;
    fn unread_char(_: &Mu, _: Tag, _: char) -> exception::Result<Option<()>>;
    fn view(_: &Mu, _: Tag) -> Tag;
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn write_byte(_: &Mu, _: Tag, _: u8) -> exception::Result<Option<()>>;
    fn write_char(_: &Mu, _: Tag, _: char) -> exception::Result<Option<()>>;
}

impl Core for Stream {
    fn gc_mark(mu: &Mu, stream: Tag) {
        let mark = Heap::mark(mu, stream).unwrap();

        if !mark {
            // GcMark(env, ptr)
            // GcMark(env, car(ptr));
            // GcMark(env, cdr(ptr));
        }
    }

    fn view(mu: &Mu, stream: Tag) -> Tag {
        let image = Self::to_image(mu, stream);
        let vec = vec![image.stream_id, image.direction, image.eof, image.unch];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn heap_size(_: &Mu, _: Tag) -> usize {
        std::mem::size_of::<Stream>()
    }

    fn is_eof(mu: &Mu, stream: Tag) -> bool {
        let image = Self::to_image(mu, stream);

        match image.direction.type_of() {
            Type::Keyword
                if image.direction.eq_(&Symbol::keyword("input"))
                    || image.direction.eq_(&Symbol::keyword("bidir")) =>
            {
                if !image.unch.null_() {
                    false
                } else {
                    !image.eof.null_()
                }
            }
            _ => !image.eof.null_(),
        }
    }

    fn clear_eof(mu: &Mu, stream: Tag) {
        let mut image = Self::to_image(mu, stream);

        image.eof = Tag::nil();
        Self::update(mu, &image, stream);
    }

    fn is_open(mu: &Mu, stream: Tag) -> bool {
        let image = Self::to_image(mu, stream);

        !image.stream_id.eq_(&Symbol::keyword("t"))
    }

    fn close(mu: &Mu, stream: Tag) {
        let mut image = Self::to_image(mu, stream);

        System::close(&mu.system, Fixnum::as_i64(image.stream_id) as usize).unwrap();

        image.stream_id = Symbol::keyword("t");
        Self::update(mu, &image, stream);
    }

    fn get_string(mu: &Mu, stream: Tag) -> exception::Result<String> {
        if !Self::is_open(mu, stream) {
            return Err(Exception::new(Condition::Open, "get-str", stream));
        }

        let image = Self::to_image(mu, stream);

        Ok(System::get_string(&mu.system, Fixnum::as_i64(image.stream_id) as usize).unwrap())
    }

    fn write(mu: &Mu, tag: Tag, _: bool, stream: Tag) -> exception::Result<()> {
        match tag.type_of() {
            Type::Stream => {
                let image = Self::to_image(mu, tag);
                match image.stream_id.type_of() {
                    Type::Keyword => {
                        <Mu as stream::Core>::write_string(mu, "#<stream: closed>", stream)
                    }
                    Type::Fixnum => <Mu as stream::Core>::write_string(
                        mu,
                        format!("#<stream: id: {}>", Fixnum::as_i64(image.stream_id)).as_str(),
                        stream,
                    ),
                    Type::Null | Type::Cons | Type::Vector => {
                        <Mu as stream::Core>::write_string(mu, "#<stream: string>", stream)
                    }
                    _ => panic!(
                        "internal: stream type inconsistency {:?}",
                        image.stream_id.type_of()
                    ),
                }
            }
            _ => panic!(),
        }
    }

    fn read_char(mu: &Mu, stream: Tag) -> exception::Result<Option<char>> {
        let mut image = Self::to_image(mu, stream);

        if !Self::is_open(mu, stream) {
            return Err(Exception::new(Condition::Open, "rd-char", stream));
        }

        if image.direction.eq_(&Symbol::keyword("output")) {
            return Err(Exception::new(Condition::Stream, "rd-char", stream));
        }

        if Self::is_eof(mu, stream) {
            return Ok(None);
        }

        match image.stream_id.type_of() {
            Type::Fixnum => {
                let stream_id = Fixnum::as_i64(image.stream_id) as usize;
                let unch = image.unch;

                if unch.null_() {
                    match System::read_byte(&mu.system, stream_id) {
                        Ok(opt) => match opt {
                            Some(byte) => Ok(Some(byte as char)),
                            None => {
                                image.eof = Symbol::keyword("t");
                                Self::update(mu, &image, stream);
                                Ok(None)
                            }
                        },
                        Err(e) => Err(e),
                    }
                } else {
                    image.unch = Tag::nil();
                    Self::update(mu, &image, stream);

                    Ok(Some(Char::as_char(mu, unch)))
                }
            }
            _ => panic!(),
        }
    }

    fn read_byte(mu: &Mu, stream: Tag) -> exception::Result<Option<u8>> {
        let mut image = Self::to_image(mu, stream);

        if !Self::is_open(mu, stream) {
            return Err(Exception::new(Condition::Open, "rd-byte", stream));
        }

        if image.direction.eq_(&Symbol::keyword("output")) {
            return Err(Exception::new(Condition::Stream, "rd-byte", stream));
        }

        if Self::is_eof(mu, stream) {
            return Ok(None);
        }

        match image.stream_id.type_of() {
            Type::Fixnum => {
                let stream_id = Fixnum::as_i64(image.stream_id) as usize;
                let unch = image.unch;

                if unch.null_() {
                    match System::read_byte(&mu.system, stream_id) {
                        Ok(opt) => match opt {
                            Some(byte) => Ok(Some(byte)),
                            None => {
                                image.eof = Symbol::keyword("t");
                                Self::update(mu, &image, stream);
                                Ok(None)
                            }
                        },
                        Err(e) => Err(e),
                    }
                } else {
                    image.unch = Tag::nil();
                    Self::update(mu, &image, stream);

                    Ok(Some(Char::as_char(mu, unch) as u8))
                }
            }
            _ => panic!(),
        }
    }

    fn unread_char(mu: &Mu, stream: Tag, ch: char) -> exception::Result<Option<()>> {
        let mut image = Self::to_image(mu, stream);

        if !Self::is_open(mu, stream) {
            return Err(Exception::new(Condition::Open, "un-char", stream));
        }

        if image.direction.eq_(&Symbol::keyword("output")) {
            return Err(Exception::new(Condition::Type, "un-char", stream));
        }

        if image.unch.null_() {
            image.unch = Char::as_tag(ch);
            Self::update(mu, &image, stream);

            Ok(None)
        } else {
            Err(Exception::new(
                Condition::Stream,
                "un-char",
                Char::as_tag(ch),
            ))
        }
    }

    fn write_char(mu: &Mu, stream: Tag, ch: char) -> exception::Result<Option<()>> {
        let image = Self::to_image(mu, stream);

        if !Self::is_open(mu, stream) {
            return Err(Exception::new(Condition::Open, "wr-char", stream));
        }

        if image.direction.eq_(&Symbol::keyword("input")) {
            return Err(Exception::new(Condition::Type, "wr-char", stream));
        }

        if Self::is_eof(mu, stream) {
            Self::clear_eof(mu, stream)
        }

        match image.stream_id.type_of() {
            Type::Fixnum => {
                let stream_id = Fixnum::as_i64(image.stream_id) as usize;
                System::write_byte(&mu.system, stream_id, ch as u8)
            }
            _ => panic!(),
        }
    }

    fn write_byte(mu: &Mu, stream: Tag, byte: u8) -> exception::Result<Option<()>> {
        let image = Self::to_image(mu, stream);

        if !Self::is_open(mu, stream) {
            return Err(Exception::new(Condition::Open, "wr-byte", stream));
        }

        if image.direction.eq_(&Symbol::keyword("input")) {
            return Err(Exception::new(Condition::Type, "wr-byte", stream));
        }

        match image.stream_id.type_of() {
            Type::Fixnum => {
                let stream_id = Fixnum::as_i64(image.stream_id) as usize;
                System::write_byte(&mu.system, stream_id, byte)
            }
            _ => panic!(
                "internal: {:?} stream state inconsistency",
                image.stream_id.type_of()
            ),
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
