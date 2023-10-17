//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu srteam functions
use crate::{
    core::{
        async_context::{AsyncContext, Core as _},
        backquote::Backquote,
        exception::{self, Condition, Exception},
        frame::Frame,
        mu::Mu,
        reader::{Core as _, Reader},
        readtable::{map_char_syntax, SyntaxType},
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
        vector::{Core as _, Vector},
    },
};

pub trait Core {
    fn read(_: &Mu, _: Tag, _: bool, _: Tag, _: bool) -> exception::Result<Tag>;
    fn write(&self, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn write_string(&self, _: String, _: Tag) -> exception::Result<()>;
}

impl Core for Mu {
    // read returns:
    //
    //     Err raise exception if I/O problem, syntax error, or end of file and !eofp
    //     Ok(eof_value) if end of file and eofp
    //     Ok(tag) if the read succeeded,
    //
    #[allow(clippy::only_used_in_recursion)]
    fn read(
        mu: &Mu,
        stream: Tag,
        eofp: bool,
        eof_value: Tag,
        recursivep: bool,
    ) -> exception::Result<Tag> {
        match Reader::read_ws(mu, stream) {
            Ok(None) => {
                if eofp {
                    return Ok(eof_value);
                } else {
                    return Err(Exception::new(Condition::Eof, "read:eo", stream));
                }
            }
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        match Stream::read_char(mu, stream) {
            Ok(None) => {
                if eofp {
                    Ok(eof_value)
                } else {
                    Err(Exception::new(Condition::Eof, "read:sm", stream))
                }
            }
            Ok(Some(ch)) => match map_char_syntax(ch) {
                Some(stype) => match stype {
                    SyntaxType::Constituent => Reader::read_atom(mu, ch, stream),
                    SyntaxType::Macro => match ch {
                        '#' => match Reader::sharp_macro(mu, stream) {
                            Ok(Some(tag)) => Ok(tag),
                            Ok(None) => Self::read(mu, stream, eofp, eof_value, recursivep),
                            Err(e) => Err(e),
                        },
                        _ => Err(Exception::new(
                            Condition::Type,
                            "read:sx",
                            Fixnum::as_tag(ch as i64),
                        )),
                    },
                    SyntaxType::Tmacro => match ch {
                        '`' => <Mu as Backquote>::bq_read(mu, false, stream, false),
                        '\'' => match Self::read(mu, stream, false, Tag::nil(), recursivep) {
                            Ok(tag) => Ok(Cons::new(
                                Symbol::keyword("quote"),
                                Cons::new(tag, Tag::nil()).evict(mu),
                            )
                            .evict(mu)),
                            Err(e) => Err(e),
                        },
                        '"' => match Vector::read(mu, '"', stream) {
                            Ok(tag) => Ok(tag),
                            Err(e) => Err(e),
                        },
                        '(' => match Cons::read(mu, stream) {
                            Ok(cons) => Ok(cons),
                            Err(e) => Err(e),
                        },
                        ')' => {
                            if recursivep {
                                Ok(mu.reader.eol)
                            } else {
                                Err(Exception::new(Condition::Syntax, "read:)", stream))
                            }
                        }
                        ';' => match Reader::read_comment(mu, stream) {
                            Ok(_) => Self::read(mu, stream, eofp, eof_value, recursivep),
                            Err(e) => Err(e),
                        },
                        ',' => Err(Exception::new(Condition::Range, "read:,", Char::as_tag(ch))),
                        _ => Err(Exception::new(
                            Condition::Range,
                            "read::@",
                            Char::as_tag(ch),
                        )),
                    },
                    _ => Err(Exception::new(Condition::Read, "read::@", Char::as_tag(ch))),
                },
                _ => Err(Exception::new(Condition::Read, "read::@", Char::as_tag(ch))),
            },
            Err(e) => Err(e),
        }
    }

    fn write(&self, tag: Tag, escape: bool, stream: Tag) -> exception::Result<()> {
        if Tag::type_of(stream) != Type::Stream {
            panic!("{:?}", Tag::type_of(stream))
        }

        match Tag::type_of(tag) {
            Type::AsyncId => AsyncContext::write(self, tag, escape, stream),
            Type::Char => Char::write(self, tag, escape, stream),
            Type::Cons => Cons::write(self, tag, escape, stream),
            Type::Fixnum => Fixnum::write(self, tag, escape, stream),
            Type::Float => Float::write(self, tag, escape, stream),
            Type::Function => Function::write(self, tag, escape, stream),
            Type::Keyword => Symbol::write(self, tag, escape, stream),
            Type::Map => Map::write(self, tag, escape, stream),
            Type::Null => Symbol::write(self, tag, escape, stream),
            Type::Stream => Stream::write(self, tag, escape, stream),
            Type::Struct => Struct::write(self, tag, escape, stream),
            Type::Symbol => Symbol::write(self, tag, escape, stream),
            Type::Vector => Vector::write(self, tag, escape, stream),
            _ => panic!(),
        }
    }

    fn write_string(&self, str: String, stream: Tag) -> exception::Result<()> {
        if Tag::type_of(stream) != Type::Stream {
            panic!("{:?}", Tag::type_of(stream))
        }
        for ch in str.chars() {
            match Stream::write_char(self, stream, ch) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

pub trait MuFunction {
    fn mu_read(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_write(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_read(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let stream = fp.argv[0];
        let eofp = fp.argv[1];
        let eof_value = fp.argv[2];

        match Tag::type_of(stream) {
            Type::Stream => match Self::read(mu, stream, !eofp.null_(), eof_value, false) {
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

        match Tag::type_of(stream) {
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
