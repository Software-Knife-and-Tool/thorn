//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu reader
use crate::{
    core::{
        exception::{self, Condition, Exception},
        frame::Frame,
        functions::Core as _,
        mu::{Core, Mu},
        reader::{Core as _, Reader},
        readtable::{map_char_syntax, SyntaxType},
        types::{Tag, Type},
    },
    types::{
        char::Char,
        cons::{Cons, ConsIter, Core as _},
        fixnum::Fixnum,
        namespace::{Core as _, Namespace},
        stream::{Core as _, Stream},
        streambuilder::StreamBuilder,
        symbol::{Core as _, Symbol},
        vector::{Core as _, Vector},
    },
};

pub trait Backquote {
    fn bq_read(_: &Mu, _: bool, _: Tag, _: bool) -> exception::Result<Tag>;
    fn bq_read_string(_: &Mu, _: String) -> exception::Result<Tag>;
    fn bq_comma(_: &Mu, _: bool, _: Tag) -> exception::Result<Tag>;
    fn bq_list(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn bq_list_elt(_: &Mu, _: Tag) -> exception::Result<Tag>;
}

impl Backquote for Mu {
    // backquote read from string:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_read_string(mu: &Mu, string: String) -> exception::Result<Tag> {
        match StreamBuilder::new().string(string).input().build(mu) {
            Ok(stream) => Reader::read(mu, stream.evict(mu), true, Tag::nil(), false),
            Err(e) => Err(e),
        }
    }

    // backquote comma:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_comma(mu: &Mu, in_list: bool, stream: Tag) -> exception::Result<Tag> {
        match Reader::read_token(mu, stream) {
            Ok(token) => match token {
                Some(mut form) => match form.chars().next().unwrap() {
                    '@' => {
                        form.remove(0);
                        if in_list {
                            Self::bq_read_string(mu, form)
                        } else {
                            Err(Exception::new(
                                Condition::Range,
                                "mu:bq_comma",
                                Vector::from_string(&form).evict(mu),
                            ))
                        }
                    }
                    _ => {
                        if in_list {
                            Ok(Cons::vlist(
                                mu,
                                &[
                                    Namespace::mu_ext_symbol(mu, "cons".to_string()),
                                    Self::bq_read_string(mu, form).unwrap(),
                                    Tag::nil(),
                                ],
                            ))
                        } else {
                            Ok(Self::bq_read_string(mu, form).unwrap())
                        }
                    }
                },
                None => Err(Exception::new(Condition::Range, "mu:bq_comma", stream)),
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
    fn bq_list_elt(mu: &Mu, expr: Tag) -> exception::Result<Tag> {
        Mu::write(mu, expr, true, mu.reader.bq_str).unwrap();

        match Stream::get_string(mu, mu.reader.bq_str) {
            Ok(string) => match StreamBuilder::new().string(string).input().build(mu) {
                Ok(stream) => {
                    let istream = stream.evict(mu);
                    match Self::bq_read(mu, false, istream, false) {
                        Ok(expr) => Ok(Cons::vlist(
                            mu,
                            &[
                                Namespace::mu_ext_symbol(mu, "cons".to_string()),
                                expr,
                                Tag::nil(),
                            ],
                        )),
                        Err(e) => Err(e),
                    }
                }
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
        let bq_fn = match Mu::map_internal(mu, "bq-append".to_string()) {
            Some(fn_) => fn_,
            None => panic!(),
        };

        match Self::bq_read(mu, true, stream, true) {
            Ok(expr) if mu.reader.eol.eq_(expr) => Ok(Tag::nil()),
            Ok(expr) => Ok(Cons::vlist(
                mu,
                &[
                    bq_fn,
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
                                Self::bq_list_elt(mu, expr)
                            } else {
                                match Tag::type_of(mu, expr) {
                                    Type::Symbol => Ok(Cons::new(
                                        Symbol::keyword("quote"),
                                        Cons::new(expr, Tag::nil()).evict(mu),
                                    )
                                    .evict(mu)),
                                    _ => Ok(expr),
                                }
                            }
                        }
                        Err(e) => Err(e),
                    },
                    SyntaxType::Macro => match ch {
                        '#' => match Reader::sharp_macro(mu, stream) {
                            Ok(Some(tag)) => Ok(tag),
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
                        '(' => match Self::bq_list(mu, stream) {
                            Ok(cons) => Ok(cons),
                            Err(e) => Err(e),
                        },
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
        let list = fp.argv[0];
        let cdr = fp.argv[1];

        fp.value = match Tag::type_of(mu, list) {
            Type::Null => cdr,
            Type::Cons => {
                let mut append = Vec::new();

                for elt in ConsIter::new(mu, list) {
                    append.push(Cons::car(mu, elt))
                }

                for elt in ConsIter::new(mu, cdr) {
                    append.push(Cons::car(mu, elt))
                }

                Cons::vlist(mu, &append)
            }
            _ => return Err(Exception::new(Condition::Type, "reader:bq_append", list)),
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
