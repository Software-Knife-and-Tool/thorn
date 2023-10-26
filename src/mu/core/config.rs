//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu config
#[allow(unused_imports)]
use crate::{
    core::types::{Tag, TagType, Type},
    types::symbol::{Core as _, Symbol},
};

#[derive(Copy, Clone)]
pub enum GcMode {
    None,
    Auto,
    Demand,
}

#[derive(Copy, Clone)]
pub struct Config {
    pub npages: usize,
    pub gcmode: GcMode,
}

pub trait Core {
    fn config(_: String) -> Option<Config>;
}

impl Core for Config {
    fn config(_str: String) -> Option<Config> {
        Some(Config {
            npages: 1024,
            gcmode: GcMode::None,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert_eq!(2 + 2, 4);
    }
}
