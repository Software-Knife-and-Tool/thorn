//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! function call frame
//!    Frame
//!    apply
//!    frame_push
//!    frame_pop
//!    frame_ref
use crate::{
    core::{
        exception::{self},
        frame::Frame,
        mu::{Core as _, Mu},
        types::Tag,
    },
    types::{
        cons::{Cons, Core as _},
        vecimage::{TypedVec, VecType},
        vector::Core as _,
    },
};

// #[allow(unused_imports)]
use futures::executor::block_on;

pub trait Core {
    fn gc_dynamic_env(&self);
    fn dynamic_push(&self, _: Tag, _: usize);
    fn dynamic_pop(&self);
    fn dynamic_ref(&self, _: usize) -> (Tag, usize);
}

impl Core for Mu {
    fn gc_dynamic_env(&self) {
        let dynamic_ref = block_on(self.dynamic.write());
        let frame_ref = block_on(self.lexical.read());

        for frame in &*dynamic_ref {
            let (id, offset) = frame;
            match frame_ref.get(id) {
                Some(vec) => {
                    let vec_ref = block_on(vec.read());
                    let frame = &vec_ref[*offset];

                    Mu::gc_mark(self, frame.func);
                    Mu::gc_mark(self, frame.value);

                    for arg in &frame.argv {
                        Mu::gc_mark(self, *arg)
                    }
                }
                None => {
                    println!("frame with function id {} not found in cache", id)
                }
            }
        }
    }

    fn dynamic_push(&self, func: Tag, offset: usize) {
        let mut dynamic_ref = block_on(self.dynamic.write());

        dynamic_ref.push((func.as_u64(), offset));
    }

    fn dynamic_pop(&self) {
        let mut dynamic_ref = block_on(self.dynamic.write());

        dynamic_ref.pop();
    }

    #[allow(dead_code)]
    fn dynamic_ref(&self, index: usize) -> (Tag, usize) {
        let dynamic_ref = block_on(self.dynamic.read());

        let (func, offset) = dynamic_ref[index];

        (Tag::from_u64(func), offset)
    }
}

pub trait MuFunction {
    fn mu_frames(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_frames(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let env_ref = block_on(mu.dynamic.read());

        let mut frames = Vec::new();

        for (func, offset) in env_ref.iter() {
            let mut argv = Vec::new();

            Frame::frame_stack_ref(mu, Tag::from_u64(*func), *offset, &mut argv);
            let vec = argv.into_iter().map(Tag::from_u64).collect();
            let values = TypedVec::<Vec<Tag>> { vec }.vec.to_vector().evict(mu);

            frames.push(Cons::new(Tag::from_u64(*func), values).evict(mu))
        }

        fp.value = Cons::vlist(mu, &frames);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dynamic() {
        assert_eq!(2 + 2, 4);
    }
}
