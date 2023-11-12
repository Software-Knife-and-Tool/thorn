//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu stream functions
#![allow(unused_braces)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::identity_op)]

use crate::{
    core::{
        exception::{self, Condition, Exception},
        frame::Frame,
        funcall::Core as _,
        mu::Mu,
        types::{Tag, Type},
    },
    system::{stream::Core as _, sys::System},
    types::{
        char::Char,
        fixnum::Fixnum,
        stream::{Core as _, Stream},
        streambuilder::StreamBuilder,
        symbol::{Core as _, Symbol},
        vector::{Core as _, Vector},
    },
};

pub trait MuFunction {
    fn mu_close(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_eof(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_flush(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_get_string(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_open(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_openp(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_read_byte(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_read_char(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_unread_char(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_write_byte(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_write_char(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Stream {
    fn mu_close(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];

        fp.value = match mu.fp_argv_check("close".to_string(), &[Type::Stream], fp) {
            Ok(_) => {
                if Self::is_open(mu, stream) {
                    Self::close(mu, stream);
                    Symbol::keyword("t")
                } else {
                    Tag::nil()
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_openp(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];

        fp.value = match mu.fp_argv_check("openp".to_string(), &[Type::Stream], fp) {
            Ok(_) => {
                if Self::is_open(mu, stream) {
                    stream
                } else {
                    Tag::nil()
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_open(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let st_type = fp.argv[0];
        let st_dir = fp.argv[1];
        let st_arg = fp.argv[2];

        fp.value = match mu.fp_argv_check(
            "open".to_string(),
            &[Type::Keyword, Type::Keyword, Type::String],
            fp,
        ) {
            Ok(_) if st_type.eq_(Symbol::keyword("file")) => {
                let arg = Vector::as_string(mu, st_arg);

                let stream = if st_dir.eq_(Symbol::keyword("input")) {
                    StreamBuilder::new().file(arg).input().build(mu)
                } else if st_dir.eq_(Symbol::keyword("output")) {
                    StreamBuilder::new().file(arg).output().build(mu)
                } else {
                    return Err(Exception::new(Condition::Type, "open", st_dir));
                };

                match stream {
                    Err(e) => return Err(e),
                    Ok(stream) => stream.evict(mu),
                }
            }
            Ok(_) if st_type.eq_(Symbol::keyword("string")) => {
                let arg = Vector::as_string(mu, st_arg);

                let stream = if st_dir.eq_(Symbol::keyword("input")) {
                    StreamBuilder::new().string(arg).input().build(mu)
                } else if st_dir.eq_(Symbol::keyword("output")) {
                    StreamBuilder::new().string(arg).output().build(mu)
                } else {
                    return Err(Exception::new(Condition::Type, "open", st_dir));
                };

                match stream {
                    Err(e) => return Err(e),
                    Ok(stream) => stream.evict(mu),
                }
            }
            Ok(_) => return Err(Exception::new(Condition::Type, "open", st_type)),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_flush(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];

        fp.value = match mu.fp_argv_check("flush".to_string(), &[Type::Stream], fp) {
            Ok(_) => {
                if Self::is_open(mu, stream) {
                    let image = Self::to_image(mu, stream);

                    if image.direction.eq_(Symbol::keyword("output"))
                        && Tag::type_of(image.stream_id) == Type::Fixnum
                    {
                        let stream_id = Fixnum::as_i64(image.stream_id) as usize;
                        System::flush(&mu.system, stream_id).unwrap()
                    }
                }

                Tag::nil()
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_eof(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];

        fp.value = match mu.fp_argv_check("eof".to_string(), &[Type::Stream], fp) {
            Ok(_) => {
                if Self::is_eof(mu, stream) {
                    Symbol::keyword("t")
                } else {
                    Tag::nil()
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_get_string(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];

        fp.value = match mu.fp_argv_check("get-str".to_string(), &[Type::Stream], fp) {
            Ok(_) => match Self::get_string(mu, stream) {
                Ok(string) => Vector::from_string(&string).evict(mu),
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_read_char(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];
        let eoferrp = fp.argv[1];
        let eof_value = fp.argv[2];

        fp.value =
            match mu.fp_argv_check("rd-char".to_string(), &[Type::Stream, Type::T, Type::T], fp) {
                Ok(_) => match Self::read_char(mu, stream) {
                    Ok(Some(ch)) => Char::as_tag(ch),
                    Ok(None) if eoferrp.null_() => eof_value,
                    Ok(None) => return Err(Exception::new(Condition::Eof, "rd-char", stream)),
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            };

        Ok(())
    }

    fn mu_read_byte(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];
        let erreofp = fp.argv[1];
        let eof_value = fp.argv[2];

        fp.value =
            match mu.fp_argv_check("rd-byte".to_string(), &[Type::Stream, Type::T, Type::T], fp) {
                Ok(_) => match Self::read_byte(mu, stream) {
                    Ok(Some(byte)) => Fixnum::as_tag(byte as i64),
                    Ok(None) if erreofp.null_() => eof_value,
                    Ok(None) => return Err(Exception::new(Condition::Eof, "rd-byte", stream)),
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            };

        Ok(())
    }

    fn mu_unread_char(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ch = fp.argv[0];
        let stream = fp.argv[1];

        fp.value = match mu.fp_argv_check("un-char".to_string(), &[Type::Char, Type::Stream], fp) {
            Ok(_) => match Self::unread_char(mu, stream, Char::as_char(mu, ch)) {
                Ok(Some(_)) => panic!(),
                Ok(None) => ch,
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_write_char(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let ch = fp.argv[0];
        let stream = fp.argv[1];

        fp.value = match mu.fp_argv_check("wr-char".to_string(), &[Type::Char, Type::Stream], fp) {
            Ok(_) => match Self::write_char(mu, stream, Char::as_char(mu, ch)) {
                Ok(_) => ch,
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_write_byte(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let byte = fp.argv[0];
        let stream = fp.argv[1];

        fp.value = match mu.fp_argv_check("wr-byte".to_string(), &[Type::Byte, Type::Stream], fp) {
            Ok(_) => match Self::write_byte(mu, stream, Fixnum::as_i64(byte) as u8) {
                Ok(_) => byte,
                Err(e) => return Err(e),
            },
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
