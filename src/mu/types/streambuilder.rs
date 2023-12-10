//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! system streams
use crate::{
    core::{
        exception::{self, Condition, Exception},
        mu::Mu,
        types::Tag,
    },
    system::{
        stream::{Core as _, STDERR, STDIN, STDOUT},
        sys::System,
    },
    types::{
        fixnum::Fixnum,
        stream::Stream,
        symbol::{Core as _, Symbol},
    },
};

pub struct StreamBuilder {
    pub file: Option<String>,
    pub string: Option<String>,
    pub input: Option<Tag>,
    pub output: Option<Tag>,
    pub bidir: Option<Tag>,
    pub stdin: Option<()>,
    pub stdout: Option<()>,
    pub errout: Option<()>,
}

impl StreamBuilder {
    pub fn new() -> Self {
        Self {
            file: None,
            string: None,
            input: None,
            output: None,
            bidir: None,
            stdin: None,
            stdout: None,
            errout: None,
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
        self.input = Some(Symbol::keyword("input"));
        self
    }

    pub fn output(&mut self) -> &mut Self {
        self.output = Some(Symbol::keyword("output"));
        self
    }

    pub fn bidir(&mut self) -> &mut Self {
        self.output = Some(Symbol::keyword("bidir"));
        self
    }

    pub fn stdin(&mut self) -> &mut Self {
        self.stdin = Some(());
        self
    }

    pub fn stdout(&mut self) -> &mut Self {
        self.stdout = Some(());
        self
    }

    pub fn errout(&mut self) -> &mut Self {
        self.errout = Some(());
        self
    }

    fn stream(stream_id: usize, direction: Tag) -> exception::Result<Stream> {
        let stream = Stream {
            stream_id: Fixnum::as_tag(stream_id as i64),
            direction,
            eof: Tag::nil(),
            unch: Tag::nil(),
        };

        Ok(stream)
    }

    pub fn build(&self, mu: &Mu) -> exception::Result<Stream> {
        match &self.file {
            Some(path) => match self.input {
                Some(input) => match System::open_input_file(&mu.system, path) {
                    Ok(id) => Self::stream(id, input),
                    Err(e) => Err(e),
                },
                None => match self.output {
                    Some(output) => match System::open_output_file(&mu.system, path) {
                        Ok(id) => Self::stream(id, output),
                        Err(e) => Err(e),
                    },
                    None => Err(Exception::new(Condition::Range, "open", Tag::nil())),
                },
            },
            None => match &self.string {
                Some(contents) => match self.input {
                    Some(input) => match System::open_input_string(&mu.system, contents) {
                        Ok(id) => Self::stream(id, input),
                        Err(e) => Err(e),
                    },
                    None => match self.output {
                        Some(output) => match System::open_output_string(&mu.system, contents) {
                            Ok(id) => Self::stream(id, output),
                            Err(e) => Err(e),
                        },
                        None => match self.bidir {
                            Some(bidir) => match System::open_bidir_string(&mu.system, contents) {
                                Ok(id) => Self::stream(id, bidir),
                                Err(e) => Err(e),
                            },
                            None => Err(Exception::new(Condition::Range, "open", Tag::nil())),
                        },
                    },
                },
                None => match self.stdin {
                    Some(_) => Self::stream(STDIN, Symbol::keyword("input")),
                    None => match self.stdout {
                        Some(_) => Self::stream(STDOUT, Symbol::keyword("output")),
                        None => match self.errout {
                            Some(_) => Self::stream(STDERR, Symbol::keyword("output")),
                            None => Err(Exception::new(Condition::Range, "open", Tag::nil())),
                        },
                    },
                },
            },
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
