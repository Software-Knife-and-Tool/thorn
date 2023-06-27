//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu reader
use crate::{
    core::{
        direct::DirectType,
        exception::{self, Condition, Exception},
        frame::Frame,
        mu::{Core, Mu},
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
    fn bq_read(_: &Mu, _: Tag, _: bool) -> exception::Result<Tag>;
}

trait Backquote {
    fn bq_comma(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn bq_list(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn bq_parse_atom(_: &Mu, _: String) -> exception::Result<Tag>;
}

impl Backquote for Mu {
    // backquote atom parser:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_parse_atom(mu: &Mu, token: String) -> exception::Result<Tag> {
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

    // backquote comma:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_comma(mu: &Mu, stream: Tag) -> exception::Result<Tag> {
        match Self::read_token(mu, stream) {
            Ok(token) => match token {
                Some(token) => match token.chars().next().unwrap() {
                    '@' => {
                        // eval and splice
                        //                        mu.eval(parse_atom(mu, token[1..].to_string()).unwrap())
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
                        Cons::new(Self::bq_parse_atom(mu, token).unwrap(), Tag::nil()).evict(mu),
                    )
                    .evict(mu)),
                },
                None => Err(Exception::new(Condition::Range, "mu:bq_comma", stream)),
            },
            Err(e) => Err(e),
        }
    }

    // backquote list:
    //
    //      return Ok(tag) for successful expansion
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn bq_list(mu: &Mu, stream: Tag) -> exception::Result<Tag> {
        let dot = Tag::to_direct('.' as u64, 1, DirectType::Byte);
        let mut cons = Tag::nil();

        loop {
            match Self::bq_read(mu, stream, true) {
                Ok(expr) => {
                    Mu::write(mu, expr, true, mu.stdout).unwrap();
                    println!();
                    if EOL.eq_(expr) {
                        return Ok(cons);
                    } else {
                        match Tag::type_of(mu, expr) {
                            Type::Symbol if dot.eq_(Symbol::name(mu, expr)) => {
                                match Self::bq_read(mu, stream, true) {
                                    Ok(_cdr) if EOL.eq_(_cdr) => (),
                                    Ok(_cdr) => match Self::bq_read(mu, stream, true) {
                                        Ok(eol) if EOL.eq_(eol) => (),
                                        Ok(_) => {
                                            return Err(Exception::new(
                                                Condition::Eof,
                                                "mu:bq_list",
                                                stream,
                                            ))
                                        }
                                        Err(e) => return Err(e),
                                    },
                                    Err(e) => return Err(e),
                                }
                            }
                            _ => {
                                cons = Cons::new(
                                    Namespace::intern(
                                        mu,
                                        mu.mu_ns,
                                        Scope::Extern,
                                        "cons".to_string(),
                                        Tag::nil(),
                                    ),
                                    Cons::new(expr, Tag::nil()).evict(mu),
                                )
                                .evict(mu);
                            }
                        }
                    }
                }
                Err(e) => return Err(e),
            }
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
    fn bq_read(mu: &Mu, stream: Tag, recursivep: bool) -> exception::Result<Tag> {
        match Self::read_ws(mu, stream) {
            Ok(None) => return Err(Exception::new(Condition::Eof, "read::read", stream)),
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        match Stream::read_char(mu, stream) {
            Ok(None) => Err(Exception::new(Condition::Eof, "mu:bq_read", stream)),
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
                        '`' => Self::bq_read(mu, stream, true),
                        ',' => Self::bq_comma(mu, stream),
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

pub trait MuFunction {
    fn mu_bq_emit(_: &Mu, fp: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_bq_emit(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let list = fp.argv[0];

        fp.value = match Tag::type_of(mu, list) {
            Type::Null => Tag::nil(),
            Type::Cons => {
                let append = Cons::new(Tag::nil(), Tag::nil());
                let tail = list;

                #[allow(clippy::while_let_loop)]
                loop {
                    match Tag::type_of(mu, Cons::cdr(mu, tail)) {
                        Type::Cons | Type::Null => match Cons::length(mu, Cons::car(mu, tail)) {
                            Some(_) => {}
                            None => {
                                return Err(Exception::new(
                                    Condition::Type,
                                    "backquote:bq_emit",
                                    Cons::car(mu, tail),
                                ))
                            }
                        },
                        _ => break,
                    }
                }

                append.evict(mu)
            }
            _ => return Err(Exception::new(Condition::Type, "mu::append", list)),
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
