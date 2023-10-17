//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu character class
use crate::{
    core::{
        direct::{DirectInfo, DirectTag, DirectType},
        exception,
        mu::Mu,
        stream,
        types::Tag,
    },
    types::{
        stream::{Core as _, Stream},
        vecimage::{TypedVec, VecType},
        vector::Core as _,
    },
};

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum Char {
    Direct(u64),
}

impl Char {
    pub fn as_char(mu: &Mu, ch: Tag) -> char {
        ((ch.data(mu) & 0xff) as u8) as char
    }

    pub fn as_tag(ch: char) -> Tag {
        DirectTag::to_direct(ch as u64, DirectInfo::Length(1), DirectType::Char)
    }
}

pub trait Core {
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn view(_: &Mu, _: Tag) -> Tag;
}

impl Core for Char {
    fn write(mu: &Mu, chr: Tag, escape: bool, stream: Tag) -> exception::Result<()> {
        let ch: u8 = (chr.data(mu) & 0xff) as u8;

        if escape {
            match <Mu as stream::Core>::write_string(mu, "#\\".to_string(), stream) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }

            let phrase = match ch {
                0x20 => "space".to_string(),
                0x9 => "tab".to_string(),
                0xa => "linefeed".to_string(),
                0xc => "page".to_string(),
                0xd => "return".to_string(),
                _ => (ch as char).to_string(),
            };

            match <Mu as stream::Core>::write_string(mu, phrase, stream) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            match Stream::write_char(mu, stream, ch as char) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }
    }

    fn view(mu: &Mu, chr: Tag) -> Tag {
        let vec = vec![chr];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::char::Char;

    #[test]
    fn as_tag() {
        match Char::as_tag('a') {
            _ => assert_eq!(true, true),
        }
    }
}
