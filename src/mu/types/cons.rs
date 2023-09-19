//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu cons class
use crate::{
    core::{
        cdirect::ConsDirect,
        direct::{DirectInfo, DirectType},
        exception::{self, Condition, Exception},
        frame::Frame,
        indirect::IndirectTag,
        mu::{Core as _, Mu},
        reader::{Core as _, Reader},
        types::{Tag, TagType, Type},
    },
    types::{
        fixnum::Fixnum,
        symbol::Symbol,
        vecimage::{TypedVec, VecType},
        vector::Core as _,
    },
};

#[cfg(feature = "async")]
use futures::executor::block_on;

#[derive(Copy, Clone)]
pub struct Cons {
    car: Tag,
    cdr: Tag,
}

impl Cons {
    pub fn new(car: Tag, cdr: Tag) -> Self {
        Cons { car, cdr }
    }

    pub fn to_image(mu: &Mu, tag: Tag) -> Self {
        match Tag::type_of(mu, tag) {
            Type::Cons => match tag {
                Tag::Indirect(main) => {
                    #[cfg(feature = "async")]
                    let heap_ref = block_on(mu.heap.read());
                    #[cfg(feature = "no-async")]
                    let heap_ref = mu.heap.borrow();

                    Cons {
                        car: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize, 8).unwrap(),
                        ),
                        cdr: Tag::from_slice(
                            heap_ref.of_length(main.offset() as usize + 8, 8).unwrap(),
                        ),
                    }
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn car(mu: &Mu, cons: Tag) -> Tag {
        match Tag::type_of(mu, cons) {
            Type::Null => cons,
            Type::Cons => match cons {
                Tag::ConsDirect(_) => ConsDirect::car(cons),
                Tag::Indirect(_) => Self::to_image(mu, cons).car,
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn cdr(mu: &Mu, cons: Tag) -> Tag {
        match Tag::type_of(mu, cons) {
            Type::Null => cons,
            Type::Cons => match cons {
                Tag::Indirect(_) => Self::to_image(mu, cons).cdr,
                Tag::ConsDirect(_) => ConsDirect::cdr(cons),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn length(mu: &Mu, cons: Tag) -> Option<usize> {
        match Tag::type_of(mu, cons) {
            Type::Null => Some(0),
            Type::Cons => {
                let mut cp = cons;
                let mut n = 0;

                loop {
                    match Tag::type_of(mu, cp) {
                        Type::Cons => {
                            n += 1;
                            cp = Self::cdr(mu, cp)
                        }
                        Type::Null => break,
                        _ => return None,
                    }
                }

                Some(n)
            }
            _ => panic!("cons::length"),
        }
    }
}

// core operations
pub trait Core {
    fn evict(&self, _: &Mu) -> Tag;
    fn vlist(_: &Mu, _: &[Tag]) -> Tag;
    fn nth(_: &Mu, _: usize, _: Tag) -> Option<Tag>;
    fn nthcdr(_: &Mu, _: usize, _: Tag) -> Option<Tag>;
    fn read(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
    fn view(_: &Mu, _: Tag) -> Tag;
}

impl Core for Cons {
    fn view(mu: &Mu, cons: Tag) -> Tag {
        let vec = vec![Self::car(mu, cons), Self::cdr(mu, cons)];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn evict(&self, mu: &Mu) -> Tag {
        match ConsDirect::cons(self.car, self.cdr) {
            Some(tag) => tag,
            None => {
                let image: &[[u8; 8]] = &[self.car.as_slice(), self.cdr.as_slice()];

                #[cfg(feature = "async")]
                let mut heap_ref = block_on(mu.heap.write());
                #[cfg(feature = "no-async")]
                let mut heap_ref = mu.heap.borrow_mut();

                let ind = IndirectTag::new()
                    .with_offset(heap_ref.alloc(image, Type::Cons as u8) as u64)
                    .with_heap_id(1)
                    .with_tag(TagType::Cons);

                Tag::Indirect(ind)
            }
        }
    }

    fn read(mu: &Mu, stream: Tag) -> exception::Result<Tag> {
        let dot = Tag::to_direct('.' as u64, DirectInfo::Length(1), DirectType::Byte);

        match Reader::read(mu, stream, false, Tag::nil(), true) {
            Ok(car) => {
                if mu.reader.eol.eq_(car) {
                    Ok(Tag::nil())
                } else {
                    match Tag::type_of(mu, car) {
                        Type::Symbol if dot.eq_(Symbol::name(mu, car)) => {
                            match Reader::read(mu, stream, false, Tag::nil(), true) {
                                Ok(cdr) if mu.reader.eol.eq_(cdr) => Ok(Tag::nil()),
                                Ok(cdr) => {
                                    match Reader::read(mu, stream, false, Tag::nil(), true) {
                                        Ok(eol) if mu.reader.eol.eq_(eol) => Ok(cdr),
                                        Ok(_) => Err(Exception::new(Condition::Eof, "car", stream)),
                                        Err(e) => Err(e),
                                    }
                                }
                                Err(e) => Err(e),
                            }
                        }
                        _ => match Self::read(mu, stream) {
                            Ok(cdr) => Ok(Cons::new(car, cdr).evict(mu)),
                            Err(e) => Err(e),
                        },
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    fn write(mu: &Mu, cons: Tag, escape: bool, stream: Tag) -> exception::Result<()> {
        let car = Self::car(mu, cons);

        mu.write_string("(".to_string(), stream).unwrap();
        mu.write(car, escape, stream).unwrap();

        let mut tail = Self::cdr(mu, cons);

        // this is ugly, but it might be worse with a for loop
        loop {
            match Tag::type_of(mu, tail) {
                Type::Cons => {
                    mu.write_string(" ".to_string(), stream).unwrap();
                    mu.write(Self::car(mu, tail), escape, stream).unwrap();
                    tail = Self::cdr(mu, tail);
                }
                _ if tail.null_() => break,
                _ => {
                    mu.write_string(" . ".to_string(), stream).unwrap();
                    mu.write(tail, escape, stream).unwrap();
                    break;
                }
            }
        }

        mu.write_string(")".to_string(), stream)
    }

    fn vlist(mu: &Mu, vec: &[Tag]) -> Tag {
        let mut list = Tag::nil();

        vec.iter()
            .rev()
            .for_each(|tag| list = Self::new(*tag, list).evict(mu));

        list
    }

    fn nth(mu: &Mu, n: usize, cons: Tag) -> Option<Tag> {
        let mut nth = n;
        let mut tail = cons;

        match Tag::type_of(mu, cons) {
            Type::Null => Some(Tag::nil()),
            Type::Cons => loop {
                match Tag::type_of(mu, tail) {
                    _ if tail.null_() => return Some(Tag::nil()),
                    Type::Cons => {
                        if nth == 0 {
                            return Some(Self::car(mu, tail));
                        }
                        nth -= 1;
                        tail = Self::cdr(mu, tail)
                    }
                    _ => {
                        return if nth != 0 { None } else { Some(tail) };
                    }
                }
            },
            _ => panic!(),
        }
    }

    fn nthcdr(mu: &Mu, n: usize, cons: Tag) -> Option<Tag> {
        let mut nth = n;
        let mut tail = cons;

        match Tag::type_of(mu, cons) {
            Type::Null => Some(Tag::nil()),
            Type::Cons => loop {
                match Tag::type_of(mu, tail) {
                    _ if tail.null_() => return Some(Tag::nil()),
                    Type::Cons => {
                        if nth == 0 {
                            return Some(tail);
                        }
                        nth -= 1;
                        tail = Self::cdr(mu, tail)
                    }
                    _ => {
                        return if nth != 0 { None } else { Some(tail) };
                    }
                }
            },
            _ => panic!(),
        }
    }
}

/// mu functions
pub trait MuFunction {
    fn mu_car(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_cdr(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_cons(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_length(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_nth(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_nthcdr(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Cons {
    fn mu_car(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let list = fp.argv[0];

        fp.value = match Tag::type_of(mu, list) {
            Type::Null => list,
            Type::Cons => Self::car(mu, list),
            _ => return Err(Exception::new(Condition::Type, "car", list)),
        };

        Ok(())
    }

    fn mu_cdr(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let list = fp.argv[0];

        fp.value = match Tag::type_of(mu, list) {
            Type::Null => list,
            Type::Cons => Self::cdr(mu, list),
            _ => return Err(Exception::new(Condition::Type, "cdr", list)),
        };

        Ok(())
    }

    fn mu_cons(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = Self::new(fp.argv[0], fp.argv[1]).evict(mu);
        Ok(())
    }

    fn mu_length(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let list = fp.argv[0];

        fp.value = match Tag::type_of(mu, list) {
            Type::Null => Fixnum::as_tag(0),
            Type::Cons => match Cons::length(mu, list) {
                Some(len) => Fixnum::as_tag(len as i64),
                None => return Err(Exception::new(Condition::Type, "length", list)),
            },
            _ => return Err(Exception::new(Condition::Type, "length", list)),
        };

        Ok(())
    }

    fn mu_nth(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let nth = fp.argv[0];
        let list = fp.argv[1];

        if Tag::type_of(mu, nth) != Type::Fixnum || Fixnum::as_i64(mu, nth) < 0 {
            return Err(Exception::new(Condition::Type, "nth", nth));
        }

        match Tag::type_of(mu, list) {
            Type::Null => {
                fp.value = Tag::nil();
                Ok(())
            }
            Type::Cons => {
                fp.value = match Self::nth(mu, Fixnum::as_i64(mu, nth) as usize, list) {
                    Some(tag) => tag,
                    None => return Err(Exception::new(Condition::Type, "nth", list)),
                };

                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "nth", list)),
        }
    }

    fn mu_nthcdr(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let nth = fp.argv[0];
        let list = fp.argv[1];

        if Tag::type_of(mu, nth) != Type::Fixnum || Fixnum::as_i64(mu, nth) < 0 {
            return Err(Exception::new(Condition::Type, "nthcdr", nth));
        }

        match Tag::type_of(mu, list) {
            Type::Null => {
                fp.value = Tag::nil();
                Ok(())
            }
            Type::Cons => {
                fp.value = match Self::nthcdr(mu, Fixnum::as_i64(mu, nth) as usize, list) {
                    Some(tag) => tag,
                    None => return Err(Exception::new(Condition::Type, "nth", list)),
                };

                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "nthcdr", list)),
        }
    }
}

/// iterator
pub struct ConsIter<'a> {
    mu: &'a Mu,
    pub cons: Tag,
}

impl<'a> ConsIter<'a> {
    pub fn new(mu: &'a Mu, cons: Tag) -> Self {
        Self { mu, cons }
    }
}

// proper lists only
impl<'a> Iterator for ConsIter<'a> {
    type Item = Tag;

    fn next(&mut self) -> Option<Self::Item> {
        match Tag::type_of(self.mu, self.cons) {
            Type::Cons => {
                let cons = self.cons;
                self.cons = Cons::cdr(self.mu, self.cons);
                Some(cons)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::types::Tag;
    use crate::types::cons::Cons;

    #[test]
    fn cons() {
        match Cons::new(Tag::nil(), Tag::nil()) {
            _ => assert_eq!(true, true),
        }
    }
}
