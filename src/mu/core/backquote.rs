//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu reader
use crate::{
    core::{
        direct::DirectType,
        exception::{self, Condition, Exception},
        mu::Mu,
        reader::{Reader as _, EOL},
        readtable::{map_char_syntax, SyntaxType},
        types::{Tag, Type},
    },
    types::{
        char::Char,
        cons::{Cons, Core as _},
        fixnum::Fixnum,
        float::Float,
        namespace::{Core as _, Namespace, Scope},
        stream::{Core as _, Stream},
        symbol::{Core as _, Symbol},
        vector::{Core as _, Vector},
    },
};

pub trait Reader {
    fn bq_append(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn bq_read(_: &Mu, _: Tag, _: bool) -> exception::Result<Tag>;
    fn bq_comma(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn bq_cons(_: &Mu, _: Tag) -> exception::Result<Tag>;
}

impl Reader for Mu {
    // backquote append:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_append(_mu: &Mu, _list: Tag) -> exception::Result<Tag> {
        Ok(Tag::nil())
    }

    // backquote comma:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_comma(mu: &Mu, stream: Tag) -> exception::Result<Tag> {
        fn parse(mu: &Mu, token: String) -> exception::Result<Tag> {
            match token.parse::<i64>() {
                Ok(fx) => Ok(Fixnum::as_tag(fx)),
                Err(_) => match token.parse::<f32>() {
                    Ok(fl) => Ok(Float::as_tag(fl)),
                    Err(_) => match Symbol::parse(mu, token) {
                        Ok(sym) => Ok(sym),
                        Err(e) => Err(e),
                    },
                },
            }
        }

        match Self::read_token(mu, stream) {
            Ok(token) => match token {
                Some(token) => match token.chars().next().unwrap() {
                    '@' => {
                        // eval and splice
                        //                        mu.eval(parse(mu, token[1..].to_string()).unwrap())
                        Ok(Tag::nil())
                    }
                    _ => Ok(Cons::new(
                        Namespace::intern(
                            mu,
                            mu.mu_ns,
                            Scope::Extern,
                            "eval".to_string(),
                            Tag::nil(),
                        ),
                        Cons::new(parse(mu, token).unwrap(), Tag::nil()).evict(mu),
                    )
                    .evict(mu)),
                },
                None => Err(Exception::new(
                    Condition::Range,
                    "read::read_char_literal",
                    stream,
                )),
            },
            Err(e) => Err(e),
        }
    }

    // backquote cons:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_cons(mu: &Mu, stream: Tag) -> exception::Result<Tag> {
        let dot = Tag::to_direct('.' as u64, 1, DirectType::Byte);

        match Self::bq_read(mu, stream, true) {
            Ok(car) => {
                if EOL.eq_(car) {
                    Ok(Tag::nil())
                } else {
                    match Tag::type_of(mu, car) {
                        Type::Symbol if dot.eq_(Symbol::name_of(mu, car)) => {
                            match Self::bq_read(mu, stream, true) {
                                Ok(cdr) if EOL.eq_(cdr) => Ok(Tag::nil()),
                                Ok(cdr) => match Self::bq_read(mu, stream, true) {
                                    Ok(eol) if EOL.eq_(eol) => Ok(cdr),
                                    Ok(_) => Err(Exception::new(Condition::Eof, "mu:car", stream)),
                                    Err(e) => Err(e),
                                },
                                Err(e) => Err(e),
                            }
                        }
                        _ => match Self::bq_cons(mu, stream) {
                            Ok(cdr) => Ok(Cons::new(
                                Namespace::intern(
                                    mu,
                                    mu.mu_ns,
                                    Scope::Extern,
                                    "cons".to_string(),
                                    Tag::nil(),
                                ),
                                Cons::new(car, Cons::new(cdr, Tag::nil()).evict(mu)).evict(mu),
                            )
                            .evict(mu)),
                            Err(e) => Err(e),
                        },
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    // bq_read returns:
    //
    //     Err raise exception if I/O problem, syntax error, or end of file
    //     Ok(tag) if the read succeeded,
    //
    #[allow(clippy::only_used_in_recursion)]
    fn bq_read(mu: &Mu, stream: Tag, recursivep: bool) -> exception::Result<Tag> {
        match Self::read_ws(mu, stream) {
            Ok(None) => return Err(Exception::new(Condition::Eof, "read::read", stream)),
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        match Stream::read_char(mu, stream) {
            Ok(None) => Err(Exception::new(Condition::Eof, "read::read", stream)),
            Ok(Some(ch)) => match map_char_syntax(ch) {
                Some(stype) => match stype {
                    SyntaxType::Constituent => match Self::read_atom(mu, ch, stream) {
                        Ok(tag) => match Tag::type_of(mu, tag) {
                            Type::Symbol => Ok(Cons::new(
                                Symbol::keyword("quote"),
                                Cons::new(tag, Tag::nil()).evict(mu),
                            )
                            .evict(mu)),
                            _ => Ok(tag),
                        },
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
                        ',' => Self::bq_comma(mu, stream),
                        '`' => Self::bq_read(mu, stream, recursivep),
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
                        '(' => match Self::bq_cons(mu, stream) {
                            Ok(cons) => Ok(cons),
                            Err(e) => Err(e),
                        },
                        ')' => {
                            if recursivep {
                                Ok(*EOL)
                            } else {
                                Err(Exception::new(Condition::Syntax, "read:read", stream))
                            }
                        }
                        ';' => match Self::read_comment(mu, stream) {
                            Ok(_) => Self::read(mu, stream, false, Tag::nil(), recursivep),
                            Err(e) => Err(e),
                        },
                        _ => Err(Exception::new(
                            Condition::Range,
                            "read::read_atom",
                            Char::as_tag(ch),
                        )),
                    },
                    _ => Err(Exception::new(
                        Condition::Read,
                        "read::read_atom",
                        Char::as_tag(ch),
                    )),
                },
                _ => Err(Exception::new(
                    Condition::Read,
                    "read::read_atom",
                    Char::as_tag(ch),
                )),
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
