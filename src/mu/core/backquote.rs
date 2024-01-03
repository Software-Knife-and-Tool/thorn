//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu reader
#[allow(unused_imports)]
use crate::{
    core::{
        compile::Compiler,
        exception::{self, Condition, Exception},
        frame::Frame,
        mu::{self, Core as _, Mu},
        reader::{Core as _, Reader},
        readtable::{map_char_syntax, SyntaxType},
        stream::{self, Core as _},
        types::{Tag, Type},
    },
    types::{
        char::Char,
        cons::{Cons, ConsIter, Core as _},
        fixnum::Fixnum,
        stream::{Core as _, Stream},
        streambuilder::StreamBuilder,
        vector::{Core as _, Vector},
    },
};

pub trait Backquote {
    fn bq_read(_: &Mu, _: bool, _: Tag, _: bool) -> exception::Result<Tag>;
    fn bq_comma(_: &Mu, _: bool, _: Tag) -> exception::Result<Tag>;
    fn bq_list(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn bq_list_element(_: &Mu, _: Tag) -> exception::Result<Tag>;
}

impl Backquote for Mu {
    // backquote comma:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_comma(mu: &Mu, in_list: bool, stream: Tag) -> exception::Result<Tag> {
        match Stream::read_char(mu, stream) {
            Ok(None) => Err(Exception::new(Condition::Eof, "comma", stream)),
            Ok(Some(ch)) => match ch {
                '@' => {
                    if in_list {
                        <Mu as stream::Core>::read(mu, stream, false, Tag::nil(), false)
                    } else {
                        Err(Exception::new(Condition::Range, "comma", stream))
                    }
                }
                ',' => Self::bq_comma(mu, in_list, stream),
                _ => {
                    Stream::unread_char(mu, stream, ch).unwrap();
                    if in_list {
                        Ok(Cons::vlist(
                            mu,
                            &[
                                mu.reader.cons,
                                match <Mu as stream::Core>::read(
                                    mu,
                                    stream,
                                    false,
                                    Tag::nil(),
                                    false,
                                ) {
                                    Ok(expr) => expr,
                                    Err(e) => return Err(e),
                                },
                                Tag::nil(),
                            ],
                        ))
                    } else {
                        <Mu as stream::Core>::read(mu, stream, false, Tag::nil(), false)
                    }
                }
            },
            Err(e) => Err(e),
        }
    }

    // backquote list element:
    //
    //      return compilable backquote function call
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_list_element(mu: &Mu, expr: Tag) -> exception::Result<Tag> {
        Mu::write(mu, expr, true, mu.reader.bq_str).unwrap();

        match Stream::get_string(mu, mu.reader.bq_str) {
            Ok(string) => match StreamBuilder::new().string(string).input().build(mu) {
                Ok(stream) => match Self::bq_read(mu, false, stream.evict(mu), false) {
                    Ok(expr) => match Tag::type_of(expr) {
                        Type::Cons | Type::Symbol => {
                            Ok(Cons::vlist(mu, &[mu.reader.cons, expr, Tag::nil()]))
                        }
                        _ => Mu::compile_quoted_list(
                            mu,
                            Cons::vlist(mu, &[Cons::vlist(mu, &[expr])]),
                        ),
                    },
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    // backquote list:
    //
    //      return compilable backquote function call
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_list(mu: &Mu, stream: Tag) -> exception::Result<Tag> {
        match Self::bq_read(mu, true, stream, true) {
            Ok(expr) if mu.reader.eol.eq_(expr) => Ok(Tag::nil()),
            Ok(expr) => Ok(Cons::vlist(
                mu,
                &[
                    mu.reader.bq_append,
                    expr,
                    match Self::bq_list(mu, stream) {
                        Ok(expr) => expr,
                        Err(e) => return Err(e),
                    },
                ],
            )),
            Err(e) => Err(e),
        }
    }

    // bq_read returns:
    //
    //     Err raise exception if I/O problem, syntax error, or end of file
    //     Ok(tag) if the read succeeded
    //
    #[allow(clippy::only_used_in_recursion)]
    fn bq_read(mu: &Mu, in_list: bool, stream: Tag, recursivep: bool) -> exception::Result<Tag> {
        match Reader::read_ws(mu, stream) {
            Ok(None) => return Err(Exception::new(Condition::Eof, "read:bq", stream)),
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        match Stream::read_char(mu, stream) {
            Ok(None) => Err(Exception::new(Condition::Eof, "read:bq", stream)),
            Ok(Some(ch)) => match map_char_syntax(ch) {
                Some(stype) => match stype {
                    SyntaxType::Constituent => match Reader::read_atom(mu, ch, stream) {
                        Ok(expr) => {
                            if in_list {
                                Self::bq_list_element(mu, expr)
                            } else {
                                match Tag::type_of(expr) {
                                    Type::Symbol => Mu::compile_quoted_list(
                                        mu,
                                        Cons::new(expr, Tag::nil()).evict(mu),
                                    ),
                                    _ => Ok(expr),
                                }
                            }
                        }
                        Err(e) => Err(e),
                    },
                    SyntaxType::Macro => match ch {
                        '#' => match Reader::sharp_macro(mu, stream) {
                            Ok(Some(expr)) => {
                                if in_list {
                                    Self::bq_list_element(mu, expr)
                                } else {
                                    Ok(expr)
                                }
                            }
                            Ok(None) => <Mu as stream::Core>::read(
                                mu,
                                stream,
                                false,
                                Tag::nil(),
                                recursivep,
                            ),
                            Err(e) => Err(e),
                        },
                        _ => Err(Exception::new(
                            Condition::Type,
                            "read::bq",
                            Fixnum::as_tag(ch as i64),
                        )),
                    },
                    SyntaxType::Tmacro => match ch {
                        '`' => Self::bq_read(mu, in_list, stream, true),
                        ',' => Self::bq_comma(mu, in_list, stream),
                        '\'' => match <Mu as stream::Core>::read(
                            mu,
                            stream,
                            false,
                            Tag::nil(),
                            recursivep,
                        ) {
                            Ok(tag) => Ok(Mu::compile_quoted_list(
                                mu,
                                Cons::new(tag, Tag::nil()).evict(mu),
                            )
                            .unwrap()),
                            Err(e) => Err(e),
                        },
                        '"' => match Vector::read(mu, '"', stream) {
                            Ok(expr) => {
                                if in_list {
                                    Self::bq_list_element(mu, expr)
                                } else {
                                    Ok(expr)
                                }
                            }
                            Err(e) => Err(e),
                        },
                        '(' => {
                            if in_list {
                                match Cons::read(mu, stream) {
                                    Ok(cons) => match Self::bq_list_element(mu, cons) {
                                        Ok(list) => Ok(list),
                                        Err(e) => Err(e),
                                    },
                                    Err(e) => Err(e),
                                }
                            } else {
                                match Self::bq_list(mu, stream) {
                                    Ok(cons) => Ok(cons),
                                    Err(e) => Err(e),
                                }
                            }
                        }
                        ')' => {
                            if recursivep {
                                Ok(mu.reader.eol)
                            } else {
                                Err(Exception::new(Condition::Syntax, "read:bq", stream))
                            }
                        }
                        ';' => match Reader::read_comment(mu, stream) {
                            Ok(_) => <Mu as stream::Core>::read(
                                mu,
                                stream,
                                false,
                                Tag::nil(),
                                recursivep,
                            ),
                            Err(e) => Err(e),
                        },
                        _ => Err(Exception::new(
                            Condition::Range,
                            "read:bq",
                            Char::as_tag(ch),
                        )),
                    },
                    _ => Err(Exception::new(Condition::Read, "read:bq", Char::as_tag(ch))),
                },
                _ => Err(Exception::new(Condition::Read, "read:bq", Char::as_tag(ch))),
            },
            Err(e) => Err(e),
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
