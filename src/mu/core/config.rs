//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu config
#[allow(unused_imports)]
use crate::{
    core::heap::GcMode,
    core::types::{Tag, TagType, Type},
    types::symbol::{Core as _, Symbol},
};

#[derive(Copy, Clone)]
pub struct Config {
    pub npages: usize,
    pub gcmode: GcMode,
}

pub trait Core {
    fn config(_: String) -> Option<Config>;
}

impl Core for Config {
    fn config(conf: String) -> Option<Config> {
        let mut config = Config {
            npages: 1024,
            gcmode: GcMode::None,
        };

        if !conf.is_empty() {
            for phrase in conf.split(',').collect::<Vec<&str>>() {
                let parse = phrase.split(':').collect::<Vec<&str>>();
                if parse.len() != 2 {
                    return None;
                } else {
                    let [name, arg] = parse[..] else { panic!() };
                    match name {
                        "npages" => match arg.parse::<usize>() {
                            Ok(n) => config.npages = n,
                            Err(_) => return None,
                        },
                        "gcmode" => {
                            config.gcmode = match arg {
                                "auto" => GcMode::Auto,
                                "none" => GcMode::None,
                                "demand" => GcMode::Demand,
                                _ => return None,
                            }
                        }
                        _ => return None,
                    }
                }
            }
        }

        Some(config)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert_eq!(2 + 2, 4);
    }
}
