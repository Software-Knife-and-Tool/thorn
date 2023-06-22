//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu direct tagged types
#![allow(unused_braces)]
#![allow(clippy::identity_op)]
use {
    crate::core::types::{Tag, TagType},
    modular_bitfield::specifiers::{B3, B56},
};

// little endian direct tag format
#[derive(Copy, Clone)]
#[bitfield]
#[repr(u64)]
pub struct DirectTag {
    #[bits = 3]
    pub tag: TagType,
    #[bits = 2]
    pub dtype: DirectType,
    pub length: B3,
    pub data: B56,
}

#[derive(BitfieldSpecifier, Copy, Clone, Eq, PartialEq)]
pub enum DirectType {
    Char = 0,
    Byte = 1,
    Keyword = 2,
    Float = 3,
}

impl Tag {
    pub const DIRECT_STR_MAX: usize = 7;

    pub fn length(&self) -> u64 {
        match self {
            Tag::Direct(tag) => tag.length() as u64,
            _ => panic!(),
        }
    }

    pub fn to_direct(data: u64, len: u8, tag: DirectType) -> Tag {
        let dir = DirectTag::new()
            .with_data(data)
            .with_length(len)
            .with_dtype(tag)
            .with_tag(TagType::Direct);

        Tag::Direct(dir)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn types() {
        assert_eq!(2 + 2, 4);
    }
}
