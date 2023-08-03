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
        frame::Frame,
        indirect::IndirectTag,
        mu::{Core as _, Mu},
        reader::{Core as _, Reader},
        types::{Tag, TagType, Type},
    },
    system::{stream::Core as _, sys::System},
    types::{
        char::Char,
        fixnum::Fixnum,
        streambuilder::StreamBuilder,
        symbol::{Core as _, Symbol},
        vecimage::{TypedVec, VecType},
        vector::{Core as _, Vector},
    },
};

// stream struct
pub struct Stream {
    pub stream_id: Tag, // system stream id (fixnum)
    pub direction: Tag, // :input | :output (keyword)
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

        let mut heap_ref = mu.heap.write().unwrap();
        Tag::Indirect(
            IndirectTag::new()
                .with_offset(heap_ref.alloc(slices, Type::Stream as u8) as u64)
                .with_heap_id(1)
                .with_tag(TagType::Indirect),
        )
    }

    pub fn to_image(mu: &Mu, tag: Tag) -> Stream {
        match Tag::type_of(mu, tag) {
            Type::Stream => match tag {
                Tag::Indirect(main) => {
                    let heap_ref = mu.heap.read().unwrap();

                    let image = Stream {
                        stream_id: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize, 8).unwrap(),
                        ),
                        direction: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize + 8, 8).unwrap(),
                        ),
                        eof: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize + 16, 8).unwrap(),
                        ),
                        unch: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize + 24, 8).unwrap(),
                        ),
                    };

                    image
                }
                _ => panic!(),
            },
            _ => panic!(),
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
            Tag::Indirect(heap) => heap.offset(),
            _ => panic!(),
        } as usize;

        let mut heap_ref = mu.heap.write().unwrap();
        heap_ref.write_image(slices, offset);
    }
}

pub trait Core {
    fn close(_: &Mu, _: Tag);
    fn is_eof(_: &Mu, _: Tag) -> bool;
    fn is_open(_: &Mu, _: Tag) -> bool;
    fn get_string(_: &Mu, _: Tag) -> exception::Result<String>;
    fn read_byte(_: &Mu, _: Tag) -> exception::Result<Option<u8>>;
    fn read_char(_: &Mu, _: Tag) -> exception::Result<Option<char>>;
    fn unread_char(_: &Mu, _: Tag, _: char) -> exception::Result<Option<()>>;
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn write_byte(_: &Mu, _: Tag, _: u8) -> exception::Result<Option<()>>;
    fn write_char(_: &Mu, _: Tag, _: char) -> exception::Result<Option<()>>;
    fn view(_: &Mu, _: Tag) -> Tag;
}

impl Core for Stream {
    fn view(mu: &Mu, stream: Tag) -> Tag {
        let image = Self::to_image(mu, stream);
        let vec = vec![image.stream_id, image.direction, image.eof, image.unch];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn is_eof(mu: &Mu, stream: Tag) -> bool {
        let image = Self::to_image(mu, stream);

        match Tag::type_of(mu, image.direction) {
            Type::Keyword if image.direction.eq_(Symbol::keyword("input")) => {
                if !image.unch.null_() {
                    false
                } else {
                    !image.eof.null_()
                }
            }
            _ => !image.eof.null_(),
        }
    }

    fn is_open(mu: &Mu, stream: Tag) -> bool {
        let image = Self::to_image(mu, stream);

        !image.stream_id.eq_(Symbol::keyword("t"))
    }

    fn close(mu: &Mu, stream: Tag) {
        let mut image = Self::to_image(mu, stream);

        System::close(&mu.system, Fixnum::as_i64(mu, image.stream_id) as usize).unwrap();

        image.stream_id = Symbol::keyword("t");
        Self::update(mu, &image, stream);
    }

    fn get_string(mu: &Mu, stream: Tag) -> exception::Result<String> {
        if !Self::is_open(mu, stream) {
            return Err(Exception::new(Condition::Open, "get-str", stream));
        }

        let image = Self::to_image(mu, stream);

        Ok(System::get_string(&mu.system, Fixnum::as_i64(mu, image.stream_id) as usize).unwrap())
    }

    fn write(mu: &Mu, tag: Tag, _: bool, stream: Tag) -> exception::Result<()> {
        match Tag::type_of(mu, tag) {
            Type::Stream => {
                let image = Self::to_image(mu, tag);
                match Tag::type_of(mu, image.stream_id) {
                    Type::Keyword => mu.write_string("#<stream: closed>".to_string(), stream),
                    Type::Fixnum => mu.write_string(
                        format!("#<stream: id: {}>", Fixnum::as_i64(mu, image.stream_id)),
                        stream,
                    ),
                    Type::Null | Type::Cons | Type::Vector => {
                        mu.write_string("#<stream: string>".to_string(), stream)
                    }
                    _ => panic!(
                        "internal: stream type inconsistency {:?}",
                        Tag::type_of(mu, image.stream_id)
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

        if image.direction.eq_(Symbol::keyword("output")) {
            return Err(Exception::new(Condition::Stream, "rd-char", stream));
        }

        if Self::is_eof(mu, stream) {
            return Ok(None);
        }

        match Tag::type_of(mu, image.stream_id) {
            Type::Fixnum => {
                let stream_id = Fixnum::as_i64(mu, image.stream_id) as usize;
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

        if image.direction.eq_(Symbol::keyword("output")) {
            return Err(Exception::new(Condition::Stream, "rd-byte", stream));
        }

        if Self::is_eof(mu, stream) {
            return Ok(None);
        }

        match Tag::type_of(mu, image.stream_id) {
            Type::Fixnum => {
                let stream_id = Fixnum::as_i64(mu, image.stream_id) as usize;
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

        if image.direction.eq_(Symbol::keyword("output")) {
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

        if image.direction.eq_(Symbol::keyword("input")) {
            return Err(Exception::new(Condition::Type, "wr-char", stream));
        }

        match Tag::type_of(mu, image.stream_id) {
            Type::Fixnum => {
                let stream_id = Fixnum::as_i64(mu, image.stream_id) as usize;
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

        if image.direction.eq_(Symbol::keyword("input")) {
            return Err(Exception::new(Condition::Type, "wr-byte", stream));
        }

        match Tag::type_of(mu, image.stream_id) {
            Type::Fixnum => {
                let stream_id = Fixnum::as_i64(mu, image.stream_id) as usize;
                System::write_byte(&mu.system, stream_id, byte)
            }
            _ => panic!(
                "internal: {:?} stream state inconsistency",
                Tag::type_of(mu, image.stream_id)
            ),
        }
    }
}

pub trait MuFunction {
    fn mu_close(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_eof(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_flush(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_get_string(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_open(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_openp(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_read(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_read_byte(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_read_char(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_unread_char(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_write(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_write_byte(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_write_char(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Stream {
    fn mu_close(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];

        fp.value = match Tag::type_of(mu, stream) {
            Type::Stream => {
                if Self::is_open(mu, stream) {
                    Self::close(mu, stream);
                    Symbol::keyword("t")
                } else {
                    Tag::nil()
                }
            }
            _ => return Err(Exception::new(Condition::Type, "close", stream)),
        };

        Ok(())
    }

    fn mu_openp(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];

        fp.value = match Tag::type_of(mu, stream) {
            Type::Stream => {
                if Self::is_open(mu, stream) {
                    stream
                } else {
                    Tag::nil()
                }
            }
            _ => return Err(Exception::new(Condition::Type, "openp", stream)),
        };

        Ok(())
    }

    fn mu_open(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let st_type = fp.argv[0];
        let st_dir = fp.argv[1];
        let st_arg = fp.argv[2];

        let arg = match Tag::type_of(mu, st_arg) {
            Type::Vector => Vector::as_string(mu, st_arg),
            _ => return Err(Exception::new(Condition::Type, "open", st_arg)),
        };

        let input = match Tag::type_of(mu, st_dir) {
            Type::Keyword if st_dir.eq_(Symbol::keyword("input")) => true,
            Type::Keyword if st_dir.eq_(Symbol::keyword("output")) => false,
            _ => return Err(Exception::new(Condition::Type, "open", st_dir)),
        };

        match Tag::type_of(mu, st_type) {
            Type::Keyword if st_type.eq_(Symbol::keyword("file")) => {
                let stream = if input {
                    StreamBuilder::new().file(arg).input().build(mu)
                } else {
                    StreamBuilder::new().file(arg).output().build(mu)
                };

                fp.value = match stream {
                    Err(e) => return Err(e),
                    Ok(stream) => stream.evict(mu),
                };

                Ok(())
            }
            Type::Keyword if st_type.eq_(Symbol::keyword("string")) => {
                let stream = if input {
                    StreamBuilder::new().string(arg).input().build(mu)
                } else {
                    StreamBuilder::new().string(arg).output().build(mu)
                };

                fp.value = match stream {
                    Err(e) => return Err(e),
                    Ok(stream) => stream.evict(mu),
                };

                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "open", st_type)),
        }
    }

    fn mu_flush(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];

        let image = Self::to_image(mu, stream);

        fp.value = Tag::nil();

        if !Self::is_open(mu, stream) {
            return Ok(());
        }

        if image.direction.eq_(Symbol::keyword("input")) {
            return Err(Exception::new(Condition::Stream, "flush", stream));
        }

        match Tag::type_of(mu, image.stream_id) {
            Type::Fixnum => {
                let stream_id = Fixnum::as_i64(mu, image.stream_id) as usize;
                System::flush(&mu.system, stream_id).unwrap()
            }
            _ => return Err(Exception::new(Condition::Type, "flush", stream)),
        }

        Ok(())
    }

    fn mu_read(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];
        let eofp = fp.argv[1];
        let eof_value = fp.argv[2];

        match Tag::type_of(mu, stream) {
            Type::Stream => match Reader::read(mu, stream, !eofp.null_(), eof_value, false) {
                Ok(tag) => {
                    fp.value = tag;
                    Ok(())
                }
                Err(e) => Err(e),
            },
            _ => Err(Exception::new(Condition::Type, "read", stream)),
        }
    }

    fn mu_write(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let value = fp.argv[0];
        let escape = fp.argv[1];
        let stream = fp.argv[2];

        match Tag::type_of(mu, stream) {
            Type::Stream => match mu.write(value, !escape.null_(), stream) {
                Ok(_) => {
                    fp.value = value;
                    Ok(())
                }
                Err(e) => Err(e),
            },
            _ => Err(Exception::new(Condition::Type, "write", stream)),
        }
    }

    fn mu_eof(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];

        match Tag::type_of(mu, stream) {
            Type::Stream => {
                fp.value = if Self::is_eof(mu, stream) {
                    Symbol::keyword("t")
                } else {
                    Tag::nil()
                };
                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "eof", stream)),
        }
    }

    fn mu_get_string(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];

        match Tag::type_of(mu, stream) {
            Type::Stream => match Self::get_string(mu, stream) {
                Ok(string) => {
                    fp.value = Vector::from_string(&string).evict(mu);
                    Ok(())
                }
                Err(e) => Err(e),
            },
            _ => Err(Exception::new(Condition::Type, "get-str", stream)),
        }
    }

    fn mu_read_char(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];
        let eoferrp = fp.argv[1];
        let eof_value = fp.argv[2];

        fp.value = match Tag::type_of(mu, stream) {
            Type::Stream => match Self::read_char(mu, stream) {
                Ok(Some(ch)) => Char::as_tag(ch),
                Ok(None) if eoferrp.null_() => eof_value,
                Ok(None) => return Err(Exception::new(Condition::Eof, "rd-char", stream)),
                Err(e) => return Err(e),
            },
            _ => return Err(Exception::new(Condition::Type, "rd-char", stream)),
        };

        Ok(())
    }

    fn mu_read_byte(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];
        let erreofp = fp.argv[1];
        let eof_value = fp.argv[2];

        fp.value = match Tag::type_of(mu, stream) {
            Type::Stream => match Self::read_byte(mu, stream) {
                Ok(Some(byte)) => Fixnum::as_tag(byte as i64),
                Ok(None) if erreofp.null_() => eof_value,
                Ok(None) => return Err(Exception::new(Condition::Eof, "rd-byte", stream)),
                Err(e) => return Err(e),
            },
            _ => return Err(Exception::new(Condition::Type, "rd-byte", stream)),
        };

        Ok(())
    }

    fn mu_unread_char(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ch = fp.argv[0];
        let stream = fp.argv[1];

        match Tag::type_of(mu, stream) {
            Type::Stream => match Self::unread_char(mu, stream, Char::as_char(mu, ch)) {
                Ok(Some(_)) => {
                    panic!()
                }
                Ok(None) => {
                    fp.value = ch;
                    Ok(())
                }
                Err(e) => Err(e),
            },
            _ => Err(Exception::new(Condition::Type, "un-char", stream)),
        }
    }

    fn mu_write_char(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ch = fp.argv[0];
        let stream = fp.argv[1];

        match Tag::type_of(mu, ch) {
            Type::Char => match Tag::type_of(mu, stream) {
                Type::Stream => match Self::write_char(mu, stream, Char::as_char(mu, ch)) {
                    Ok(_) => {
                        fp.value = ch;
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                _ => Err(Exception::new(Condition::Type, "wr-char", stream)),
            },
            _ => Err(Exception::new(Condition::Type, "wr-char", stream)),
        }
    }

    fn mu_write_byte(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let byte = fp.argv[0];
        let stream = fp.argv[1];

        match Tag::type_of(mu, byte) {
            Type::Fixnum if Fixnum::as_i64(mu, byte) < 256 => match Tag::type_of(mu, stream) {
                Type::Stream => {
                    match Self::write_byte(mu, stream, Fixnum::as_i64(mu, byte) as u8) {
                        Ok(_) => {
                            fp.value = byte;
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                }
                _ => Err(Exception::new(Condition::Type, "wr-byte", stream)),
            },
            Type::Fixnum => Err(Exception::new(Condition::Range, "wr-byte", byte)),
            _ => Err(Exception::new(Condition::Type, "wr-byte", byte)),
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
