//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! system interface
use {
    crate::system::stream::Stream,
    std::{cell::RefCell, time::SystemTime},
};

// system state
pub struct System {
    pub stream_info: RefCell<Vec<Stream>>,
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

impl System {
    pub fn new() -> Self {
        System {
            stream_info: RefCell::new(Vec::new()),
        }
    }

    pub fn real_time() -> std::result::Result<usize, ()> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(elapsed) => Ok(elapsed.as_secs() as usize),
            Err(_) => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::system::sys::System;

    #[test]
    fn system() {
        match System::new() {
            _ => assert_eq!(true, true),
        }
    }
}
