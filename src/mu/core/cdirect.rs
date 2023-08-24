//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! cdr code
#![allow(unused_braces)]
#![allow(clippy::identity_op)]
use {
    crate::core::types::{Tag, TagType},
    modular_bitfield::specifiers::{B1, B30},
};

#[derive(Copy, Clone)]
pub struct ConsDirect {
    data: u64,
}

#[derive(Copy, Clone)]
#[bitfield]
#[repr(u64)]
pub struct ConsDirectTag {
    #[bits = 3]
    pub tag: TagType,
    pub cdr: B30,
    pub car: B30,
    msb: B1, // always 0
}

impl ConsDirect {
    // can tag be sign extended to 64 from 30 bits?
    pub fn from_tag(tag: Tag) -> Option<u32> {
        let u64_ = tag.as_u64();

        let mask_30: u64 = 0x3fffffff;
        let mask_34: u64 = 0x3ffffffff;
        let up_34: u64 = u64_ >> 30;
        let bot_30: u32 = (u64_ & mask_30).try_into().unwrap();
        let msb: u64 = u64_ >> 29 & 1;

        match msb {
            0 if up_34 == 0 && msb == 0 => Some(bot_30),
            1 if up_34 == mask_34 && msb == 1 => Some(bot_30),
            _ => None,
        }
    }

    pub fn cons(car: Tag, cdr: Tag) -> Option<Tag> {
        match Self::from_tag(car) {
            Some(car_) => match Self::from_tag(cdr) {
                Some(cdr_) => {
                    let cons = ConsDirectTag::new()
                        .with_tag(TagType::ConsDirect)
                        .with_cdr(cdr_)
                        .with_car(car_)
                        .with_msb(0);

                    Some(Tag::ConsDirect(cons))
                }
                None => None,
            },
            None => None,
        }
    }

    pub fn car(cons: Tag) -> Tag {
        let mut u64_: u64 = ConsDirectTag::from(cons.as_u64()).car() as u64;
        let sign = (u64_ >> 29) & 1;

        let mask_34: u64 = 0x3ffffffff;
        if sign != 0 {
            u64_ |= mask_34 << 30;
        }

        Tag::from_u64(u64_)
    }

    pub fn cdr(cons: Tag) -> Tag {
        let mut u64_: u64 = ConsDirectTag::from(cons.as_u64()).cdr() as u64;
        let sign = (u64_ >> 29) & 1;

        let mask_34: u64 = 0x3ffffffff;
        if sign != 0 {
            u64_ |= mask_34 << 30;
        }

        Tag::from_u64(u64_)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn cdirect() {
        assert_eq!(2 + 2, 4);
    }
}
