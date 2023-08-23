//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! cdr code
#![allow(unused_braces)]
#![allow(clippy::identity_op)]
use {
    crate::core::{
        mu::Mu,
        types::{Tag, TagType},
    },
    modular_bitfield::specifiers::{B1, B60},
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
    pub data: B60,
    msb: B1, // always 0
}

impl ConsDirect {
    // can tag be sign extended from 30 bits?
    pub fn u32_from_tag(tag: Tag) -> Option<u32> {
        let u64_ = tag.as_u64();

        let mask_30: u64 = 0x3fffffff;
        let mask_34: u64 = 0x3ffffffff;
        let up_34: u64 = u64_ >> 30;
        let bot_30: u32 = (u64_ & mask_30).try_into().unwrap();
        let msb: u64 = u64_ >> 29 & 1;

        match msb {
            0 if up_34 == 0 => Some(bot_30),
            1 if up_34 == mask_34 => Some(bot_30),
            _ => None,
        }
    }

    pub fn cons(car: Tag, cdr: Tag) -> Option<Tag> {
        match Self::u32_from_tag(car) {
            Some(car_) => match Self::u32_from_tag(cdr) {
                Some(cdr_) => {
                    let data = (car_ as u64) << 34 | (cdr_ as u64) & 0x3fffffff;

                    if (data >> 60) != 0 || (data >> 59) & 1 != 0 {
                        return None;
                    }
                    /*
                    if (data >> 60) & 1 == 1 {
                        return None;
                    }
                     */

                    let cons = ConsDirectTag::new()
                        .with_data(data)
                        .with_tag(TagType::ConsDirect)
                        .with_msb(0);

                    Some(Tag::ConsDirect(cons))
                }
                None => None,
            },
            None => None,
        }
    }

    pub fn car(mu: &Mu, cons: Tag) -> Tag {
        let bits: i64 = (cons.data(mu) >> 34).try_into().unwrap();

        Tag::from_u64(bits as u64)
    }

    pub fn cdr(mu: &Mu, cons: Tag) -> Tag {
        let bits: i64 = (cons.data(mu) << 33).try_into().unwrap();

        Tag::from_u64((bits >> 33) as u64)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn cdirect() {
        assert_eq!(2 + 2, 4);
    }
}
