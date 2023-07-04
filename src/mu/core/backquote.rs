//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu reader
use crate::{
    core::{
        exception::{self, Condition, Exception},
        frame::Frame,
        functions::Core as _,
        mu::{Core, Mu},
        reader::{Reader as _, EOL},
        readtable::{map_char_syntax, SyntaxType},
        types::{Tag, Type},
    },
    types::{
        char::Char,
        cons::{Cons, ConsIter, Core as _},
        fixnum::Fixnum,
        namespace::{Core as _, Namespace, Scope},
        stream::{Core as _, Stream},
        streambuilder::StreamBuilder,
        symbol::{Core as _, Symbol},
        vector::{Core as _, Vector},
    },
};

pub trait Reader {
    fn bq_read(_: &Mu, _: bool, _: Tag, _: bool) -> exception::Result<Tag>;
}

trait Backquote {
    fn bq_read_string(_: &Mu, _: String) -> exception::Result<Tag>;
    fn bq_comma(_: &Mu, _: bool, _: Tag) -> exception::Result<Tag>;
    fn bq_list(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn bq_list_elt(_: &Mu, _: Tag) -> exception::Result<Tag>;
}

impl Backquote for Mu {
    // backquote atom parser:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_read_string(mu: &Mu, string: String) -> exception::Result<Tag> {
        match StreamBuilder::new().string(string).input().build(mu) {
            Ok(stream) => Self::read(mu, stream.evict(mu), true, Tag::nil(), false),
            Err(e) => Err(e),
        }
    }

    // backquote comma:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_comma(mu: &Mu, in_list: bool, stream: Tag) -> exception::Result<Tag> {
        match Self::read_token(mu, stream) {
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
                                    Namespace::intern(
                                        mu,
                                        mu.mu_ns,
                                        Scope::Extern,
                                        "cons".to_string(),
                                        Tag::nil(),
                                    ),
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
        match StreamBuilder::new()
            .string("".to_string())
            .output()
            .build(mu)
        {
            Ok(stream) => {
                let ostream = stream.evict(mu);
                Mu::write(mu, expr, true, ostream).unwrap();
                match Stream::get_string(mu, ostream) {
                    Ok(string) => match StreamBuilder::new().string(string).input().build(mu) {
                        Ok(stream) => {
                            let istream = stream.evict(mu);
                            match Self::bq_read(mu, false, istream, false) {
                                Ok(expr) => Ok(Cons::vlist(
                                    mu,
                                    &[
                                        Namespace::intern(
                                            mu,
                                            mu.mu_ns,
                                            Scope::Extern,
                                            "cons".to_string(),
                                            Tag::nil(),
                                        ),
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
            Ok(expr) if EOL.eq_(expr) => Ok(Tag::nil()),
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
}

impl Reader for Mu {
    // bq_read returns:
    //
    //     Err raise exception if I/O problem, syntax error, or end of file
    //     Ok(tag) if the read succeeded,
    //
    #[allow(clippy::only_used_in_recursion)]
    fn bq_read(mu: &Mu, in_list: bool, stream: Tag, recursivep: bool) -> exception::Result<Tag> {
        match Self::read_ws(mu, stream) {
            Ok(None) => return Err(Exception::new(Condition::Eof, "reader:bq_read", stream)),
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        match Stream::read_char(mu, stream) {
            Ok(None) => Err(Exception::new(Condition::Eof, "reader:bq_read", stream)),
            Ok(Some(ch)) => match map_char_syntax(ch) {
                Some(stype) => match stype {
                    SyntaxType::Constituent => match Self::read_atom(mu, ch, stream) {
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
                        '#' => match Self::sharp_macro(mu, stream) {
                            Ok(Some(tag)) => Ok(tag),
                            Ok(None) => Self::read(mu, stream, false, Tag::nil(), recursivep),
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
                        '(' => match Self::bq_list(mu, stream) {
                            Ok(cons) => Ok(cons),
                            Err(e) => Err(e),
                        },
                        ')' => {
                            if recursivep {
                                Ok(*EOL)
                            } else {
                                Err(Exception::new(Condition::Syntax, "reader:bq_read", stream))
                            }
                        }
                        ';' => match Self::read_comment(mu, stream) {
                            Ok(_) => Self::read(mu, stream, false, Tag::nil(), recursivep),
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
