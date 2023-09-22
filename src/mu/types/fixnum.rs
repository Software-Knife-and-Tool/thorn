//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu fixnum type
use crate::{
    core::{
        exception::{self, Condition, Exception, Result},
        frame::Frame,
        mu::{Core as _, Mu},
        types::{Tag, TagType, Type},
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
    // u64 to tag
    pub fn as_tag(fx: i64) -> Tag {
        Tag::Fixnum(fx << 3 | TagType::Fixnum as i64)
    }

    // tag as i64
    pub fn as_i64(mu: &Mu, tag: Tag) -> i64 {
        match Tag::type_of(tag) {
            Type::Fixnum => Tag::data(&tag, mu) as i64,
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
        mu.write_string(Self::as_i64(mu, tag).to_string(), stream)
    }

    fn view(mu: &Mu, fx: Tag) -> Tag {
        let vec = vec![fx];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }
}

pub trait MuFunction {
    fn mu_fxadd(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxsub(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxor(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxand(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxdiv(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxlt(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fxmul(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Fixnum {
    fn mu_fxadd(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        match Tag::type_of(fx0) {
            Type::Fixnum => match Tag::type_of(fx1) {
                Type::Fixnum => {
                    fp.value = Self::as_tag(Fixnum::as_i64(mu, fx0) + Fixnum::as_i64(mu, fx1));
                    Ok(())
                }
                _ => Err(Exception::new(Condition::Type, "fx-add", fx1)),
            },
            _ => Err(Exception::new(Condition::Type, "fx-add", fx0)),
        }
    }

    fn mu_fxsub(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        match Tag::type_of(fx0) {
            Type::Fixnum => match Tag::type_of(fx1) {
                Type::Fixnum => {
                    fp.value = Self::as_tag(Fixnum::as_i64(mu, fx0) - Fixnum::as_i64(mu, fx1));
                    Ok(())
                }
                _ => Err(Exception::new(Condition::Type, "fx-sub", fx1)),
            },
            _ => Err(Exception::new(Condition::Type, "fx-sub", fx0)),
        }
    }

    fn mu_fxmul(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        match Tag::type_of(fx0) {
            Type::Fixnum => match Tag::type_of(fx1) {
                Type::Fixnum => {
                    fp.value = Self::as_tag(Fixnum::as_i64(mu, fx0) * Fixnum::as_i64(mu, fx1));
                    Ok(())
                }
                _ => Err(Exception::new(Condition::Type, "fx-mul", fx1)),
            },
            _ => Err(Exception::new(Condition::Type, "fx-mul", fx0)),
        }
    }

    fn mu_fxlt(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        match Tag::type_of(fx0) {
            Type::Fixnum => match Tag::type_of(fx1) {
                Type::Fixnum => {
                    fp.value = if Fixnum::as_i64(mu, fx0) < Fixnum::as_i64(mu, fx1) {
                        Symbol::keyword("t")
                    } else {
                        Tag::nil()
                    };

                    Ok(())
                }
                _ => Err(Exception::new(Condition::Type, "fx-lt", fx1)),
            },
            _ => Err(Exception::new(Condition::Type, "fx-lt", fx0)),
        }
    }

    fn mu_fxdiv(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        match Tag::type_of(fx0) {
            Type::Fixnum => match Tag::type_of(fx1) {
                Type::Fixnum => {
                    let dividend = Fixnum::as_i64(mu, fx0);
                    let divisor = Fixnum::as_i64(mu, fx1);

                    if divisor == 0 {
                        return Err(Exception::new(Condition::ZeroDivide, "fx-div", fx0));
                    }

                    fp.value = Self::as_tag(dividend / divisor);
                    Ok(())
                }
                _ => Err(Exception::new(Condition::Type, "fx-div", fx1)),
            },
            _ => Err(Exception::new(Condition::Type, "fx-div", fx0)),
        }
    }

    fn mu_fxand(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        match Tag::type_of(fx0) {
            Type::Fixnum => match Tag::type_of(fx1) {
                Type::Fixnum => {
                    fp.value = Self::as_tag(Fixnum::as_i64(mu, fx0) & Fixnum::as_i64(mu, fx1));
                    Ok(())
                }
                _ => Err(Exception::new(Condition::Type, "logand", fx1)),
            },
            _ => Err(Exception::new(Condition::Type, "logand", fx0)),
        }
    }

    fn mu_fxor(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let fx0 = fp.argv[0];
        let fx1 = fp.argv[1];

        match Tag::type_of(fx0) {
            Type::Fixnum => match Tag::type_of(fx1) {
                Type::Fixnum => {
                    fp.value = Self::as_tag(Fixnum::as_i64(mu, fx0) | Fixnum::as_i64(mu, fx1));
                    Ok(())
                }
                _ => Err(Exception::new(Condition::Type, "logor", fx1)),
            },
            _ => Err(Exception::new(Condition::Type, "logor", fx0)),
        }
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
