//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! system streams
use {
    crate::system::stream::Stream,
    std::{cell::RefCell, collections::VecDeque, fs},
};

pub struct StreamBuilder {
    pub file: Option<String>,
    pub string: Option<String>,
    pub input: Option<()>,
    pub output: Option<()>,
    pub bidir: Option<()>,
}

impl StreamBuilder {
    pub fn new() -> Self {
        Self {
            file: None,
            string: None,
            input: None,
            output: None,
            bidir: None,
        }
    }

    pub fn file(&mut self, path: String) -> &mut Self {
        self.file = Some(path);
        self
    }

    pub fn string(&mut self, contents: String) -> &mut Self {
        self.string = Some(contents);
        self
    }

    pub fn input(&mut self) -> &mut Self {
        self.input = Some(());
        self
    }

    pub fn output(&mut self) -> &mut Self {
        self.output = Some(());
        self
    }

    pub fn bidir(&mut self) -> &mut Self {
        self.bidir = Some(());
        self
    }

    pub fn build(&self) -> Option<Stream> {
        match &self.file {
            Some(path) => match self.input {
                Some(_) => match fs::File::open(path) {
                    Ok(file) => Some(Stream::File(RefCell::new(file))),
                    Err(_) => None,
                },
                None => match self.output {
                    Some(_) => match fs::File::create(path) {
                        Ok(file) => Some(Stream::File(RefCell::new(file))),
                        Err(_) => None,
                    },
                    None => None,
                },
            },
            None => self.string.as_ref().map(|contents| {
                Stream::String(RefCell::new(VecDeque::from(contents.as_bytes().to_vec())))
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::system::{stream::Stream, streambuilder::StreamBuilder};

    #[test]
    fn stream_builder() {
        let stream = StreamBuilder::new()
            .string("hello".to_string())
            .input()
            .build();

        match stream {
            Some(stream) => match stream {
                Stream::String(_) => assert_eq!(true, true),
                _ => assert_eq!(true, false),
            },
            None => assert_eq!(true, false),
        }
    }
}
