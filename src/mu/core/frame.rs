//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! function call frame
//!    Frame
//!    apply
//!    frame_push
//!    frame_pop
//!    frame_ref
use {
    crate::{
        core::{
            exception::{self, Condition, Exception},
            mu::{Core as _, Mu},
            types::{Tag, Type},
        },
        types::{
            cons::{Cons, ConsIter, Core as _},
            fixnum::Fixnum,
            function::Function,
            struct_::{Core as _, Struct},
            symbol::{Core as _, Symbol},
            vecimage::{TypedVec, VecType, VectorIter},
            vector::{Core as _, Vector},
        },
    },
    std::sync::RwLock,
};

pub struct Frame {
    pub func: Tag,
    pub argv: Vec<Tag>,
    pub value: Tag,
}

impl Frame {
    fn to_tag(&self, mu: &Mu) -> Tag {
        let mut vec: Vec<Tag> = vec![self.func];

        for arg in &self.argv {
            vec.push(*arg)
        }

        Struct::new(mu, "frame".to_string(), vec).evict(mu)
    }

    fn from_tag(mu: &Mu, tag: Tag) -> Self {
        match Tag::type_of(mu, tag) {
            Type::Struct => {
                let stype = Struct::stype(mu, tag);
                let frame = Struct::vector(mu, tag);

                let func = Vector::ref_(mu, frame, 0).unwrap();

                match Tag::type_of(mu, func) {
                    Type::Function => {
                        if !stype.eq_(Symbol::keyword("frame")) {
                            panic!()
                        }

                        let mut args = Vec::new();

                        for arg in VectorIter::new(mu, frame).skip(1) {
                            args.push(arg)
                        }

                        Frame {
                            func,
                            argv: args,
                            value: Tag::nil(),
                        }
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }

    // dynamic environment
    fn env_push(mu: &Mu, func: Tag, offset: usize) {
        let mut dynamic_ref = mu.dynamic.write().unwrap();

        dynamic_ref.push((func.as_u64(), offset));
    }

    fn env_pop(mu: &Mu) {
        let mut dynamic_ref = mu.dynamic.write().unwrap();

        dynamic_ref.pop();
    }

    #[allow(dead_code)]
    fn dynamic_ref(mu: &Mu, index: usize) -> (Tag, usize) {
        let dynamic_ref = mu.dynamic.read().unwrap();
        let (func, offset) = dynamic_ref[index];

        (Tag::from_u64(func), offset)
    }

    // frame stacks
    fn frame_stack_push(self, mu: &Mu) {
        let id = Function::id(mu, self.func).as_u64();
        let mut stack_ref = mu.lexical.write().unwrap();

        if let std::collections::hash_map::Entry::Vacant(e) = stack_ref.entry(id) {
            e.insert(RwLock::new(vec![self]));
        } else {
            let mut vec_ref = stack_ref[&id].write().unwrap();
            vec_ref.push(self);
        }
    }

    fn frame_stack_pop(mu: &Mu, id: Tag) {
        let stack_ref = mu.lexical.read().unwrap();
        let mut vec_ref = stack_ref[&id.as_u64()].write().unwrap();

        vec_ref.pop();
    }

    fn frame_stack_ref(mu: &Mu, id: Tag, offset: usize, argv: &mut Vec<u64>) {
        let stack_ref = mu.lexical.read().unwrap();
        let vec_ref = stack_ref[&id.as_u64()].read().unwrap();

        for value in &vec_ref[offset].argv {
            argv.push(value.as_u64())
        }
    }

    fn frame_stack_len(mu: &Mu, id: Tag) -> Option<usize> {
        let stack_ref = mu.lexical.read().unwrap();

        if stack_ref.contains_key(&id.as_u64()) {
            let vec_ref = stack_ref[&id.as_u64()].read().unwrap();
            Some(vec_ref.len())
        } else {
            None
        }
    }

    // frame reference
    fn frame_ref(mu: &Mu, id: u64, offset: usize) -> Option<Tag> {
        let stack_ref = mu.lexical.read().unwrap();
        let vec_ref = stack_ref[&id].read().unwrap();

        Some(vec_ref[vec_ref.len() - 1].argv[offset])
    }

    // apply
    pub fn apply(mut self, mu: &Mu, func: Tag) -> exception::Result<Tag> {
        match Tag::type_of(mu, func) {
            Type::Symbol => {
                if Symbol::is_unbound(mu, func) {
                    Err(Exception::new(Condition::Unbound, "frame::apply", func))
                } else {
                    self.apply(mu, Symbol::value(mu, func))
                }
            }
            Type::Function => match Tag::type_of(mu, Function::form(mu, func)) {
                Type::Null => Ok(Tag::nil()),
                Type::Fixnum => {
                    let nreqs = Fixnum::as_i64(mu, Function::nreq(mu, func)) as usize;
                    let nargs = self.argv.len();

                    if nargs != nreqs {
                        return Err(Exception::new(Condition::Arity, "frame::apply", func));
                    }

                    let fn_off = Fixnum::as_i64(mu, Function::form(mu, func)) as usize;
                    let fnc = mu.functions[fn_off];

                    match fnc(mu, &mut self) {
                        Ok(_) => Ok(self.value),
                        Err(e) => Err(e),
                    }
                }
                Type::Cons => {
                    let nreqs = Fixnum::as_i64(mu, Function::nreq(mu, func)) as usize;
                    let nargs = self.argv.len();

                    if nargs != nreqs {
                        return Err(Exception::new(Condition::Arity, "frame::apply", func));
                    }

                    let mut value = Tag::nil();
                    let offset =
                        Self::frame_stack_len(mu, Function::id(mu, self.func)).unwrap_or(0);

                    Self::env_push(mu, self.func, offset);
                    self.frame_stack_push(mu);

                    for cons in ConsIter::new(mu, Function::form(mu, func)) {
                        value = match mu.eval(Cons::car(mu, cons)) {
                            Ok(value) => value,
                            Err(e) => return Err(e),
                        };
                    }

                    Self::frame_stack_pop(mu, Function::id(mu, func));
                    Self::env_pop(mu);

                    Ok(value)
                }
                _ => Err(Exception::new(Condition::Type, "frame::apply::car", func)),
            },
            _ => Err(Exception::new(Condition::Type, "frame::apply", func)),
        }
    }
}

pub trait MuFunction {
    fn mu_frames(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fr_pop(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fr_push(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fr_ref(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Frame {
    fn mu_frames(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let env_ref = mu.dynamic.read().unwrap();
        let mut frames = Vec::new();

        for (func, offset) in env_ref.iter() {
            let mut argv = Vec::new();

            Self::frame_stack_ref(
                mu,
                Function::id(mu, Tag::from_u64(*func)),
                *offset,
                &mut argv,
            );
            let vec = argv.into_iter().map(Tag::from_u64).collect();
            let values = TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu);

            frames.push(Cons::new(Tag::from_u64(*func), values).evict(mu))
        }

        fp.value = Cons::vlist(mu, &frames);
        Ok(())
    }

    fn mu_fr_pop(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = fp.argv[0];

        match Tag::type_of(mu, fp.value) {
            Type::Function => Self::frame_stack_pop(mu, fp.value),
            _ => return Err(Exception::new(Condition::Type, "mu:fr-pop", fp.value)),
        }

        Ok(())
    }

    fn mu_fr_push(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = fp.argv[0];

        match Tag::type_of(mu, fp.value) {
            Type::Vector => Self::from_tag(mu, fp.value).frame_stack_push(mu),
            _ => return Err(Exception::new(Condition::Type, "mu:fr-push", fp.value)),
        }

        Ok(())
    }

    fn mu_fr_ref(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let frame = fp.argv[0];
        let offset = fp.argv[1];

        match Tag::type_of(mu, frame) {
            Type::Fixnum => match Tag::type_of(mu, offset) {
                Type::Fixnum => match Frame::frame_ref(
                    mu,
                    Fixnum::as_i64(mu, frame) as u64,
                    Fixnum::as_i64(mu, offset) as usize,
                ) {
                    Some(tag) => {
                        fp.value = tag;
                        Ok(())
                    }
                    None => Err(Exception::new(Condition::Type, "mu:lex-ref", frame)),
                },
                _ => Err(Exception::new(Condition::Type, "mu:lex-ref", offset)),
            },
            _ => Err(Exception::new(Condition::Type, "mu:lex-ref", frame)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn frame() {
        assert_eq!(2 + 2, 4);
    }
}
