//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu fixnum type
use crate::{
    core::{
        direct::{DirectInfo, DirectTag, DirectType, ExtType},
        exception::{self, Condition, Exception, Result},
        frame::Frame,
        funcall::Core as _,
        mu::Mu,
        stream,
        types::{Tag, Type},
    },
    types::{
        symbol::{Core as _, Symbol},
        vecimage::{TypedVec, VecType},
        vector::Core as _,
    },
};

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum Fixnum {
    Direct(u64),
}

impl Fixnum {
    // range checking
    pub fn is_i56(u56: u64) -> bool {
        match u56 & (1 << 56) {
            0 => (u56 >> 56) == 0,
            _ => ((u56 as i64) >> 56) == -1,
        }
    }

    // tag i64
    pub fn as_tag(fx: i64) -> Tag {
        if !Self::is_i56(fx as u64) {
            panic!("fixnum overflow")
        }

        DirectTag::to_direct(
            (fx & 0x00ffffffffffffff) as u64,
            DirectInfo::ExtType(ExtType::Fixnum),
            DirectType::Ext,
        )
    }

    // untag fixnum
    pub fn as_i64(tag: Tag) -> i64 {
        match Tag::type_of(tag) {
            Type::Fixnum => (tag.as_u64() as i64) >> 8,
            _ => panic!(),
        }
    }
}

pub trait Core {
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> Result<()>;
    fn view(_: &Mu, _: Tag) -> Tag;
}

impl Core for Fixnum {
    fn write(mu: &Mu, tag: Tag, _escape: bool, stream: Tag) -> Result<()> {
        <Mu as stream::Core>::write_string(mu, Self::as_i64(tag).to_string(), stream)
    }

    fn view(mu: &Mu, fx: Tag) -> Tag {
        let vec = vec![fx];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }
}

pub trait MuFunction {
    fn mu_fxadd(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxash(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxsub(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_logor(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_logand(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxdiv(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxlt(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxmul(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Fixnum {
    fn mu_fxash(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match mu.fp_argv_check("fx-ash".to_string(), &[Type::Fixnum, Type::Fixnum], fp) {
            Ok(_) => {
                let value = Self::as_i64(fp.argv[0]);
                let shift = Self::as_i64(fp.argv[1]);

                let result = if shift < 0 {
                    value >> shift.abs()
                } else {
                    value << shift
                };

                Self::as_tag(result)
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_fxadd(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        fp.value = match mu.fp_argv_check("fx-add".to_string(), &[Type::Fixnum, Type::Fixnum], fp) {
            Ok(_) => match Self::as_i64(fx0).checked_add(Self::as_i64(fx1)) {
                Some(sum) => {
                    if Self::is_i56(sum as u64) {
                        Self::as_tag(sum)
                    } else {
                        return Err(Exception::new(Condition::Over, "fx-add", fx0));
                    }
                }
                None => return Err(Exception::new(Condition::Over, "fx-add", fx1)),
            },
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_fxsub(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        fp.value = match mu.fp_argv_check("fx-sub".to_string(), &[Type::Fixnum, Type::Fixnum], fp) {
            Ok(_) => match Self::as_i64(fx0).checked_sub(Self::as_i64(fx1)) {
                Some(diff) => {
                    if Self::is_i56(diff as u64) {
                        Self::as_tag(diff)
                    } else {
                        return Err(Exception::new(Condition::Over, "fx-sub", fx1));
                    }
                }
                None => return Err(Exception::new(Condition::Over, "fx-sub", fx1)),
            },
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_fxmul(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        fp.value = match mu.fp_argv_check("fx-mul".to_string(), &[Type::Fixnum, Type::Fixnum], fp) {
            Ok(_) => match Self::as_i64(fx0).checked_mul(Self::as_i64(fx1)) {
                Some(prod) => {
                    if Self::is_i56(prod as u64) {
                        Self::as_tag(prod)
                    } else {
                        return Err(Exception::new(Condition::Over, "fx-mul", fx1));
                    }
                }
                None => return Err(Exception::new(Condition::Over, "fx-mul", fx1)),
            },
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_fxdiv(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        fp.value = match mu.fp_argv_check("fx-div".to_string(), &[Type::Fixnum, Type::Fixnum], fp) {
            Ok(_) => {
                if Self::as_i64(fx1) == 0 {
                    return Err(Exception::new(Condition::ZeroDivide, "fx-div", fx0));
                }

                match Self::as_i64(fx0).checked_div(Self::as_i64(fx1)) {
                    Some(div) => {
                        if Self::is_i56(div as u64) {
                            Self::as_tag(div)
                        } else {
                            return Err(Exception::new(Condition::Over, "fx-div", fx1));
                        }
                    }
                    None => return Err(Exception::new(Condition::Over, "fx-div", fx1)),
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_logand(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        fp.value = match mu.fp_argv_check("logand".to_string(), &[Type::Fixnum, Type::Fixnum], fp) {
            Ok(_) => Self::as_tag(Self::as_i64(fx0) & Self::as_i64(fx1)),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_logor(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        fp.value = match mu.fp_argv_check("logor".to_string(), &[Type::Fixnum, Type::Fixnum], fp) {
            Ok(_) => Self::as_tag(Self::as_i64(fx0) | Self::as_i64(fx1)),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_fxlt(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        fp.value = match mu.fp_argv_check("fx-lt".to_string(), &[Type::Fixnum, Type::Fixnum], fp) {
            Ok(_) => {
                if Self::as_i64(fx0) < Self::as_i64(fx1) {
                    Symbol::keyword("t")
                } else {
                    Tag::nil()
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::fixnum::Fixnum;

    #[test]
    fn as_tag() {
        match Fixnum::as_tag(0) {
            _ => assert_eq!(true, true),
        }
    }
}
