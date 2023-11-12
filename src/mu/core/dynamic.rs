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
        mu::Mu,
        types::Tag,
    },
    types::{
        cons::{Cons, Core as _},
        vecimage::{TypedVec, VecType},
        vector::Core as _,
    },
};

use futures::executor::block_on;

impl Mu {
    pub fn dynamic_push(&self, func: Tag, offset: usize) {
        let mut dynamic_ref = block_on(self.dynamic.write());

        dynamic_ref.push((func.as_u64(), offset));
    }

    pub fn dynamic_pop(&self) {
        let mut dynamic_ref = block_on(self.dynamic.write());

        dynamic_ref.pop();
    }

    #[allow(dead_code)]
    pub fn dynamic_ref(&self, index: usize) -> (Tag, usize) {
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
