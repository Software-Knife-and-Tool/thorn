//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu direct tagged types
#![allow(unused_braces)]
#![allow(clippy::identity_op)]
use {
    crate::core::types::{Tag, TagType},
    modular_bitfield::specifiers::{B3, B56},
    num_enum::TryFromPrimitive,
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
    pub info: B3,
    pub data: B56,
}

#[derive(BitfieldSpecifier, Copy, Clone, Eq, PartialEq)]
pub enum DirectType {
    Char = 0,
    Byte = 1,
    Keyword = 2,
    Ext = 3,
}

#[derive(Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum ExtType {
    Float = 0,
    AsyncId = 1,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DirectInfo {
    Length(usize),
    ExtType(ExtType),
}

impl Tag {
    pub const DIRECT_STR_MAX: usize = 7;

    pub fn length(&self) -> usize {
        match self {
            Tag::Direct(tag) => tag.info() as usize,
            _ => panic!(),
        }
    }

    pub fn ext(&self) -> u64 {
        match self {
            Tag::Direct(tag) => tag.info() as u64,
            _ => panic!(),
        }
    }

    pub fn to_direct(data: u64, info: DirectInfo, tag: DirectType) -> Tag {
        let info: u8 = match info {
            DirectInfo::Length(usize_) => usize_ as u8,
            DirectInfo::ExtType(type_) => type_ as u8,
        };

        let dir = DirectTag::new()
            .with_data(data)
            .with_info(info)
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
