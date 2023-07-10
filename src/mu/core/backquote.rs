//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu reader
use crate::{
    core::{
        compile::Compiler,
        exception::{self, Condition, Exception},
        frame::Frame,
        mu::{Core, Mu},
        reader::{Core as _, Reader},
        readtable::{map_char_syntax, SyntaxType},
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
            Ok(None) => Err(Exception::new(Condition::Eof, "mu:bq_comma", stream)),
            Ok(Some(ch)) => match ch {
                '@' => {
                    if in_list {
                        Reader::read(mu, stream, false, Tag::nil(), false)
                    } else {
                        Err(Exception::new(Condition::Range, "mu:bq_comma", stream))
                    }
                }
                _ => {
                    Stream::unread_char(mu, stream, ch).unwrap();
                    if in_list {
                        Ok(Cons::vlist(
                            mu,
                            &[
                                mu.reader.cons,
                                match Reader::read(mu, stream, false, Tag::nil(), false) {
                                    Ok(expr) => expr,
                                    Err(e) => return Err(e),
                                },
                                Tag::nil(),
                            ],
                        ))
                    } else {
                        Reader::read(mu, stream, false, Tag::nil(), false)
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
                    Ok(expr) => match Tag::type_of(mu, expr) {
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
    //     Ok(tag) if the read succeeded,
    //
    #[allow(clippy::only_used_in_recursion)]
    fn bq_read(mu: &Mu, in_list: bool, stream: Tag, recursivep: bool) -> exception::Result<Tag> {
        match Reader::read_ws(mu, stream) {
            Ok(None) => return Err(Exception::new(Condition::Eof, "reader:bq_read", stream)),
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        match Stream::read_char(mu, stream) {
            Ok(None) => Err(Exception::new(Condition::Eof, "reader:bq_read", stream)),
            Ok(Some(ch)) => match map_char_syntax(ch) {
                Some(stype) => match stype {
                    SyntaxType::Constituent => match Reader::read_atom(mu, ch, stream) {
                        Ok(expr) => {
                            if in_list {
                                Self::bq_list_element(mu, expr)
                            } else {
                                match Tag::type_of(mu, expr) {
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
                            Ok(None) => Reader::read(mu, stream, false, Tag::nil(), recursivep),
                            Err(e) => Err(e),
                        },
                        _ => Err(Exception::new(
                            Condition::Type,
                            "read::read",
                            Fixnum::as_tag(ch as i64),
                        )),
                    },
                    SyntaxType::Tmacro => match ch {
                        '`' => Self::bq_read(mu, in_list, stream, true),
                        ',' => Self::bq_comma(mu, in_list, stream),
                        '\'' => match Reader::read(mu, stream, false, Tag::nil(), recursivep) {
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
                                Err(Exception::new(Condition::Syntax, "reader:bq_read", stream))
                            }
                        }
                        ';' => match Reader::read_comment(mu, stream) {
                            Ok(_) => Reader::read(mu, stream, false, Tag::nil(), recursivep),
                            Err(e) => Err(e),
                        },
                        _ => Err(Exception::new(
                            Condition::Range,
                            "reader:bq_read",
                            Char::as_tag(ch),
                        )),
                    },
                    _ => Err(Exception::new(
                        Condition::Read,
                        "reader:bq_read",
                        Char::as_tag(ch),
                    )),
                },
                _ => Err(Exception::new(
                    Condition::Read,
                    "reader:bq_read",
                    Char::as_tag(ch),
                )),
            },
            Err(e) => Err(e),
        }
    }
}

pub trait MuFunction {
    fn mu_bq_append(_: &Mu, fp: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_bq_append(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let list1 = fp.argv[0];
        let list2 = fp.argv[1];

        fp.value = match Tag::type_of(mu, list1) {
            Type::Null | Type::Cons => {
                let mut append = Vec::new();

                for elt in ConsIter::new(mu, list1) {
                    append.push(Cons::car(mu, elt))
                }

                match Tag::type_of(mu, list2) {
                    Type::Null | Type::Cons => {
                        for elt in ConsIter::new(mu, list2) {
                            append.push(Cons::car(mu, elt))
                        }

                        Cons::vlist(mu, &append)
                    }
                    _ => return Err(Exception::new(Condition::Type, "reader:bq_append", list2)),
                }
            }
            _ => return Err(Exception::new(Condition::Type, "reader:bq_append", list1)),
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
