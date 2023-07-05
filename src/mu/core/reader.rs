//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu reader
use crate::{
    core::{
        backquote::Backquote,
        direct::DirectType,
        exception::{self, Condition, Exception},
        mu::Mu,
        readtable::{map_char_syntax, SyntaxType},
        types::{Tag, Type},
    },
    types::{
        char::Char,
        cons::{Cons, Core as _},
        fixnum::Fixnum,
        float::Float,
        stream::{Core as _, Stream},
        streambuilder::StreamBuilder,
        struct_::{Core as _, Struct},
        symbol::{Core as _, Symbol},
        vector::{Core as _, Vector},
    },
};

pub struct Reader {
    pub eol: Tag,
    pub bq_str: Tag,
}

//
// read functions return:
//
//     Ok(Some(())) if the function succeeded,
//     Ok(None) if end of file
//     Err if stream or syntax error
//     errors propagate out of read()
//
pub trait Core {
    fn new() -> Self;
    fn build(&self, _: &Mu) -> Self;
    fn read_atom(_: &Mu, _: char, _: Tag) -> exception::Result<Tag>;
    fn read(_: &Mu, _: Tag, _: bool, _: Tag, _: bool) -> exception::Result<Tag>;
    fn read_block_comment(_: &Mu, _: Tag) -> exception::Result<Option<()>>;
    fn read_char_literal(_: &Mu, _: Tag) -> exception::Result<Option<Tag>>;
    fn read_comment(_: &Mu, _: Tag) -> exception::Result<Option<()>>;
    fn read_ws(_: &Mu, _: Tag) -> exception::Result<Option<()>>;
    fn sharp_macro(_: &Mu, _: Tag) -> exception::Result<Option<Tag>>;
    fn read_token(_: &Mu, _: Tag) -> exception::Result<Option<String>>;
}

impl Core for Reader {
    //
    // reader creation:
    //
    fn new() -> Self {
        Reader {
            eol: Tag::to_direct(0, 0, DirectType::Keyword),
            bq_str: Tag::nil(),
        }
    }

    fn build(&self, mu: &Mu) -> Self {
        Reader {
            eol: self.eol,
            bq_str: StreamBuilder::new()
                .string("".to_string())
                .output()
                .build(mu)
                .unwrap()
                .evict(mu),
        }
    }

    //
    // read whitespace:
    //
    //    leave non-ws char at the head of the stream
    //    return None on end of file (not an error)
    //    return Err exception for stream error
    //    return Ok(Some(())) for ws consumed
    //
    fn read_ws(mu: &Mu, stream: Tag) -> exception::Result<Option<()>> {
        loop {
            match Stream::read_char(mu, stream) {
                Ok(Some(ch)) => {
                    if let Some(stype) = map_char_syntax(ch) {
                        match stype {
                            SyntaxType::Whitespace => (),
                            _ => {
                                Stream::unread_char(mu, stream, ch).unwrap();
                                break;
                            }
                        }
                    }
                }
                Ok(None) => return Ok(None),
                Err(e) => return Err(e),
            }
        }

        Ok(Some(()))
    }

    // read comment till end of line:
    //
    //     return Err exception for stream error
    //     return Ok(Some(())) for comment consumed
    //
    fn read_comment(mu: &Mu, stream: Tag) -> exception::Result<Option<()>> {
        loop {
            match Stream::read_char(mu, stream) {
                Ok(Some(ch)) => {
                    if ch == '\n' {
                        break;
                    }
                }
                Ok(None) => {
                    return Err(Exception::new(Condition::Eof, "read::read_comment", stream))
                }
                Err(e) => return Err(e),
            }
        }

        Ok(Some(()))
    }

    // read block comment
    //
    //     leave non-ws char at the head of the stream
    //     return Err exception for stream error
    //     return Ok(Some(())) for comment consumed
    //
    fn read_block_comment(mu: &Mu, stream: Tag) -> exception::Result<Option<()>> {
        loop {
            match Stream::read_char(mu, stream) {
                Ok(Some(ch)) => {
                    if ch == '|' {
                        match Stream::read_char(mu, stream) {
                            Ok(Some(ch)) => {
                                if ch == '#' {
                                    break;
                                }
                            }
                            Ok(None) => {
                                return Err(Exception::new(
                                    Condition::Eof,
                                    "read::read_block_comment",
                                    stream,
                                ))
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(None) => {
                    return Err(Exception::new(
                        Condition::Eof,
                        "read::read_block_comment",
                        stream,
                    ))
                }
                Err(e) => return Err(e),
            }
        }

        Ok(Some(()))
    }

    // read token
    //
    //     return Err exception for stream error
    //     return Ok(Some(String))
    //
    fn read_token(mu: &Mu, stream: Tag) -> exception::Result<Option<String>> {
        let mut token = String::new();

        loop {
            match Stream::read_char(mu, stream) {
                Ok(Some(ch)) => match map_char_syntax(ch) {
                    Some(stype) => match stype {
                        SyntaxType::Constituent => token.push(ch),
                        SyntaxType::Whitespace | SyntaxType::Tmacro => {
                            Stream::unread_char(mu, stream, ch).unwrap();
                            break;
                        }
                        _ => {
                            return Err(Exception::new(
                                Condition::Range,
                                "read::read_token",
                                stream,
                            ))
                        }
                    },
                    None => {
                        return Err(Exception::new(Condition::Range, "read::read_token", stream))
                    }
                },
                Ok(None) => {
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(Some(token))
    }

    // read symbol or numeric literal:
    //
    //      leave non-ws char at the head of the stream
    //      return Some(tag) for successful read
    //      return Err exception for stream I/O error or unexpected eof
    //
    fn read_atom(mu: &Mu, ch: char, stream: Tag) -> exception::Result<Tag> {
        let mut token = String::new();

        token.push(ch);
        loop {
            match Stream::read_char(mu, stream) {
                Ok(Some(ch)) => match map_char_syntax(ch) {
                    Some(stype) => match stype {
                        SyntaxType::Constituent => token.push(ch),
                        SyntaxType::Whitespace | SyntaxType::Tmacro => {
                            Stream::unread_char(mu, stream, ch).unwrap();
                            break;
                        }
                        _ => {
                            return Err(Exception::new(
                                Condition::Range,
                                "read::read_atom",
                                Char::as_tag(ch),
                            ))
                        }
                    },
                    None => {
                        return Err(Exception::new(
                            Condition::Range,
                            "read::read_atom",
                            Char::as_tag(ch),
                        ))
                    }
                },
                Ok(None) => {
                    break;
                }
                Err(e) => return Err(e),
            }
        }

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

    // read_char_literal returns:
    //
    //     Err exception if I/O problem or syntax error
    //     Ok(tag) if the read succeeded,
    //
    fn read_char_literal(mu: &Mu, stream: Tag) -> exception::Result<Option<Tag>> {
        match Stream::read_char(mu, stream) {
            Ok(Some(ch)) => match Stream::read_char(mu, stream) {
                Ok(Some(ch_)) => match map_char_syntax(ch_) {
                    Some(stype) => match stype {
                        SyntaxType::Constituent => {
                            Stream::unread_char(mu, stream, ch_).unwrap();
                            match Self::read_token(mu, stream) {
                                Ok(Some(str)) => {
                                    let phrase = ch.to_string() + &str;
                                    match phrase.as_str() {
                                        "tab" => Ok(Some(Char::as_tag('\t'))),
                                        "linefeed" => Ok(Some(Char::as_tag('\n'))),
                                        "space" => Ok(Some(Char::as_tag(' '))),
                                        "page" => Ok(Some(Char::as_tag('\x0c'))),
                                        "return" => Ok(Some(Char::as_tag('\r'))),
                                        _ => Err(Exception::new(
                                            Condition::Range,
                                            "read::read_char_literal",
                                            stream,
                                        )),
                                    }
                                }
                                Ok(None) => Err(Exception::new(
                                    Condition::Eof,
                                    "read::read_char_literal",
                                    stream,
                                )),
                                Err(e) => Err(e),
                            }
                        }
                        _ => {
                            Stream::unread_char(mu, stream, ch_).unwrap();
                            Ok(Some(Char::as_tag(ch)))
                        }
                    },
                    None => Err(Exception::new(
                        Condition::Syntax,
                        "read::read_char_literal",
                        stream,
                    )),
                },
                Ok(None) => Ok(Some(Char::as_tag(ch))),
                Err(e) => Err(e),
            },
            Ok(None) => Err(Exception::new(
                Condition::Eof,
                "read::read_char_literal",
                stream,
            )),
            Err(e) => Err(e),
        }
    }

    // sharp_macro returns:
    //
    //     Err exception if I/O problem or syntax error
    //     Ok(tag) if the read succeeded,
    //
    fn sharp_macro(mu: &Mu, stream: Tag) -> exception::Result<Option<Tag>> {
        match Stream::read_char(mu, stream) {
            Ok(Some(ch)) => match ch {
                ':' => match Stream::read_char(mu, stream) {
                    Ok(Some(ch)) => match Self::read_atom(mu, ch, stream) {
                        Ok(atom) => match Tag::type_of(mu, atom) {
                            Type::Symbol => Ok(Some(atom)),
                            _ => Err(Exception::new(Condition::Type, "read::sharp_macro", stream)),
                        },
                        Err(e) => Err(e),
                    },
                    Ok(None) => Err(Exception::new(Condition::Eof, "read::sharp_macro", stream)),
                    Err(e) => Err(e),
                },
                '|' => match Self::read_block_comment(mu, stream) {
                    Ok(_) => Ok(None),
                    Err(e) => Err(e),
                },
                '\\' => Self::read_char_literal(mu, stream),
                'S' | 's' => match Struct::read(mu, stream) {
                    Ok(tag) => Ok(Some(tag)),
                    Err(e) => Err(e),
                },
                '(' => match Vector::read(mu, '(', stream) {
                    Ok(tag) => Ok(Some(tag)),
                    Err(e) => Err(e),
                },
                'x' => match Self::read_token(mu, stream) {
                    Ok(token) => match token {
                        Some(hex) => match i64::from_str_radix(&hex, 16) {
                            Ok(fx) => Ok(Some(Fixnum::as_tag(fx))),
                            Err(_) => Err(Exception::new(
                                Condition::Syntax,
                                "read::sharp_macro",
                                Char::as_tag(ch),
                            )),
                        },
                        None => panic!(),
                    },
                    Err(_) => Err(Exception::new(
                        Condition::Syntax,
                        "read::sharp_macro",
                        Char::as_tag(ch),
                    )),
                },
                _ => Err(Exception::new(
                    Condition::Type,
                    "read::sharp_macro",
                    Char::as_tag(ch),
                )),
            },
            Ok(None) => Err(Exception::new(Condition::Eof, "read::sharp_macro", stream)),
            Err(e) => Err(e),
        }
    }

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
        match Self::read_ws(mu, stream) {
            Ok(None) => {
                if eofp {
                    return Ok(eof_value);
                } else {
                    return Err(Exception::new(Condition::Eof, "read::read", stream));
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
                    Err(Exception::new(Condition::Eof, "read::read", stream))
                }
            }
            Ok(Some(ch)) => match map_char_syntax(ch) {
                Some(stype) => match stype {
                    SyntaxType::Constituent => Self::read_atom(mu, ch, stream),
                    SyntaxType::Macro => match ch {
                        '#' => match Self::sharp_macro(mu, stream) {
                            Ok(Some(tag)) => Ok(tag),
                            Ok(None) => Self::read(mu, stream, eofp, eof_value, recursivep),
                            Err(e) => Err(e),
                        },
                        _ => Err(Exception::new(
                            Condition::Type,
                            "read::read",
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
                                Err(Exception::new(Condition::Syntax, "read:read", stream))
                            }
                        }
                        ';' => match Self::read_comment(mu, stream) {
                            Ok(_) => Self::read(mu, stream, eofp, eof_value, recursivep),
                            Err(e) => Err(e),
                        },
                        ',' => Err(Exception::new(
                            Condition::Range,
                            "reader::comma_invalid",
                            Char::as_tag(ch),
                        )),
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
