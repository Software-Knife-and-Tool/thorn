//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu function type
use crate::{
    core::{
        exception,
        heap::Core as _,
        indirect::IndirectTag,
        mu::{Core as _, Mu},
        stream,
        types::{Tag, TagType, Type},
    },
    types::{
        fixnum::Fixnum,
        symbol::Symbol,
        vecimage::{TypedVec, VecType},
        vector::{Core as _, Vector},
    },
};

use futures::executor::block_on;

#[derive(Copy, Clone)]
pub struct Function {
    id: Tag,    // frame id, nil or a symbol
    arity: Tag, // fixnum # of required arguments
    form: Tag,  // cons body or fixnum native table offset
}

impl Function {
    pub fn new(arity: Tag, form: Tag, id: Tag) -> Self {
        Function { arity, form, id }
    }

    pub fn evict(&self, mu: &Mu) -> Tag {
        let image: &[[u8; 8]] = &[
            self.arity.as_slice(),
            self.form.as_slice(),
            self.id.as_slice(),
        ];

        let mut heap_ref = block_on(mu.heap.write());
        let ind = IndirectTag::new()
            .with_image_id(heap_ref.alloc(image, Type::Function as u8) as u64)
            .with_heap_id(1)
            .with_tag(TagType::Function);

        Tag::Indirect(ind)
    }

    pub fn to_image(mu: &Mu, tag: Tag) -> Self {
        match Tag::type_of(tag) {
            Type::Function => match tag {
                Tag::Indirect(main) => {
                    let heap_ref = block_on(mu.heap.read());

                    Function {
                        arity: Tag::from_slice(
                            heap_ref.of_length(main.image_id() as usize, 8).unwrap(),
                        ),
                        form: Tag::from_slice(
                            heap_ref.of_length(main.image_id() as usize + 8, 8).unwrap(),
                        ),
                        id: Tag::from_slice(
                            heap_ref
                                .of_length(main.image_id() as usize + 16, 8)
                                .unwrap(),
                        ),
                    }
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn arity(mu: &Mu, func: Tag) -> Tag {
        Self::to_image(mu, func).arity
    }

    pub fn form(mu: &Mu, func: Tag) -> Tag {
        Self::to_image(mu, func).form
    }

    pub fn id(mu: &Mu, func: Tag) -> Tag {
        Self::to_image(mu, func).id
    }
}

pub trait Core {
    fn gc_mark(_: &Mu, _: Tag);
    fn heap_size(_: &Mu, _: Tag) -> usize;
    fn view(_: &Mu, _: Tag) -> Tag;
    fn write(_: &Mu, _: Tag, _: bool, _: Tag) -> exception::Result<()>;
}

impl Core for Function {
    fn gc_mark(mu: &Mu, function: Tag) {
        let mark = mu.mark(function).unwrap();

        if !mark {
            Mu::gc_mark(mu, function);
            Mu::gc_mark(mu, Self::form(mu, function))
        }
    }

    fn view(mu: &Mu, func: Tag) -> Tag {
        let vec = vec![
            Self::arity(mu, func),
            Self::form(mu, func),
            Self::id(mu, func),
        ];

        TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu)
    }

    fn heap_size(mu: &Mu, fn_: Tag) -> usize {
        match Tag::type_of(Self::form(mu, fn_)) {
            Type::Null | Type::Cons => std::mem::size_of::<Function>(),
            Type::Fixnum => std::mem::size_of::<Function>(),
            _ => panic!(),
        }
    }

    fn write(mu: &Mu, func: Tag, _: bool, stream: Tag) -> exception::Result<()> {
        match Tag::type_of(func) {
            Type::Function => {
                let nreq = Fixnum::as_i64(Function::arity(mu, func));
                let form = Function::form(mu, func);

                let desc = match Tag::type_of(form) {
                    Type::Cons | Type::Null => {
                        (":lambda".to_string(), format!("{:x}", form.as_u64()))
                    }
                    Type::Fixnum => {
                        let name = Function::id(mu, func);
                        (
                            format!("mu:{}", Vector::as_string(mu, Symbol::name(mu, name))),
                            form.as_u64().to_string(),
                        )
                    }
                    _ => {
                        panic!()
                    }
                };

                <Mu as stream::Core>::write_string(
                    mu,
                    format!("#<:function {} [req:{nreq}, tag:{}]>", desc.0, desc.1),
                    stream,
                )
            }
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::types::Tag;
    use crate::types::fixnum::Fixnum;
    use crate::types::function::Function;

    #[test]
    fn as_tag() {
        match Function::new(Fixnum::as_tag(0), Tag::nil(), Tag::nil()) {
            _ => assert_eq!(true, true),
        }
    }
}
