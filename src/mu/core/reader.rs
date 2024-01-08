//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu reader
use crate::{
    core::{
        direct::{DirectInfo, DirectTag, DirectType},
        exception::{self, Condition, Exception},
        mu::Mu,
        namespace::Namespace,
        readtable::{map_char_syntax, SyntaxType},
        types::{Tag, Type},
    },
    types::{
        char::Char,
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
    pub cons: Tag,
    pub bq_append: Tag,
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
    fn read_block_comment(_: &Mu, _: Tag) -> exception::Result<Option<()>>;
    fn read_char_literal(_: &Mu, _: Tag) -> exception::Result<Option<Tag>>;
    fn read_comment(_: &Mu, _: Tag) -> exception::Result<Option<()>>;
    fn read_ws(_: &Mu, _: Tag) -> exception::Result<Option<()>>;
    fn sharp_macro(_: &Mu, _: Tag) -> exception::Result<Option<Tag>>;
    fn read_token(_: &Mu, _: Tag) -> exception::Result<Option<String>>;
}

impl Core for Reader {
    fn new() -> Self {
        Reader {
            cons: Tag::nil(),
            eol: DirectTag::to_direct(0, DirectInfo::Length(0), DirectType::Keyword),
            bq_str: Tag::nil(),
            bq_append: Tag::nil(),
        }
    }

    fn build(&self, mu: &Mu) -> Self {
        Reader {
            eol: self.eol,
            cons: Namespace::intern_symbol(mu, mu.mu_ns, "cons".to_string(), Tag::nil()),
            bq_str: StreamBuilder::new()
                .string("".to_string())
                .output()
                .build(mu)
                .unwrap()
                .evict(mu),
            bq_append: mu.append_,
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
                Ok(None) => return Err(Exception::new(Condition::Eof, "read:;", stream)),
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
                                return Err(Exception::new(Condition::Eof, "read:#|", stream))
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(None) => return Err(Exception::new(Condition::Eof, "read:#|", stream)),
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
                        _ => return Err(Exception::new(Condition::Range, "read:tk", stream)),
                    },
                    None => return Err(Exception::new(Condition::Range, "read:tk", stream)),
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
                                "read:at",
                                Char::as_tag(ch),
                            ))
                        }
                    },
                    None => {
                        return Err(Exception::new(
                            Condition::Range,
                            "read:at",
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
            Ok(fx) => {
                if Fixnum::is_i56(fx as u64) {
                    Ok(Fixnum::as_tag(fx))
                } else {
                    Err(Exception::new(
                        Condition::Over,
                        "read:at",
                        Vector::from_string(&token).evict(mu),
                    ))
                }
            }
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
                                        _ => {
                                            Err(Exception::new(Condition::Range, "read:ch", stream))
                                        }
                                    }
                                }
                                Ok(None) => Err(Exception::new(Condition::Eof, "read:ch", stream)),
                                Err(e) => Err(e),
                            }
                        }
                        _ => {
                            Stream::unread_char(mu, stream, ch_).unwrap();
                            Ok(Some(Char::as_tag(ch)))
                        }
                    },
                    None => Err(Exception::new(Condition::Syntax, "read:ch", stream)),
                },
                Ok(None) => Ok(Some(Char::as_tag(ch))),
                Err(e) => Err(e),
            },
            Ok(None) => Err(Exception::new(Condition::Eof, "read:ch", stream)),
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
                        Ok(atom) => match atom.type_of() {
                            Type::Symbol => Ok(Some(atom)),
                            _ => Err(Exception::new(Condition::Type, "read:#", stream)),
                        },
                        Err(e) => Err(e),
                    },
                    Ok(None) => Err(Exception::new(Condition::Eof, "read:#", stream)),
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
                            Ok(fx) => {
                                if Fixnum::is_i56(fx as u64) {
                                    Ok(Some(Fixnum::as_tag(fx)))
                                } else {
                                    Err(Exception::new(
                                        Condition::Over,
                                        "read:#x",
                                        Vector::from_string(&hex).evict(mu),
                                    ))
                                }
                            }
                            Err(_) => Err(Exception::new(
                                Condition::Syntax,
                                "read:#",
                                Char::as_tag(ch),
                            )),
                        },
                        None => panic!(),
                    },
                    Err(_) => Err(Exception::new(
                        Condition::Syntax,
                        "read:#",
                        Char::as_tag(ch),
                    )),
                },
                _ => Err(Exception::new(Condition::Type, "read:#", Char::as_tag(ch))),
            },
            Ok(None) => Err(Exception::new(Condition::Eof, "read:#", stream)),
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
