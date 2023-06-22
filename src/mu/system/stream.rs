//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! system streams
use {
    crate::core::{
        exception::{self, Condition, Exception},
        types::Tag,
    },
    crate::system::{streambuilder::StreamBuilder, sys::System},
    std::{
        cell::{Ref, RefCell, RefMut},
        collections::VecDeque,
        fs,
        io::{Read, Write},
        str,
    },
};

pub enum Stream {
    File(RefCell<fs::File>),
    String(RefCell<VecDeque<u8>>),
}

// we cannot possibly ever have this many streams open
pub const STDIN: usize = 0x80000000;
pub const STDOUT: usize = 0x80000001;
pub const STDERR: usize = 0x80000002;

pub trait Core {
    fn close(_: &System, _: usize) -> Option<()>;
    fn flush(_: &System, _: usize) -> Option<()>;
    fn get_string(_: &System, _: usize) -> Option<String>;
    fn is_file(system: &System, _: usize) -> Option<bool>;
    fn is_string(system: &System, _: usize) -> Option<bool>;
    fn open_file(_: &System, _: &str, _: bool) -> exception::Result<usize>;
    fn open_string(_: &System, _: &str, _: bool) -> exception::Result<usize>;
    fn read_byte(_: &System, _: usize) -> exception::Result<Option<u8>>;
    fn write_byte(_: &System, _: usize, _: u8) -> exception::Result<Option<()>>;
}

impl Core for System {
    fn is_file(system: &System, index: usize) -> Option<bool> {
        match index {
            STDIN | STDOUT | STDERR => Some(false),
            _ => {
                let stream_info_ref: Ref<Vec<Stream>> = system.stream_info.borrow();
                if index >= stream_info_ref.len() {
                    None
                } else {
                    match stream_info_ref.get(index).unwrap() {
                        Stream::File(_) => Some(true),
                        _ => Some(false),
                    }
                }
            }
        }
    }

    fn is_string(system: &System, index: usize) -> Option<bool> {
        match index {
            STDIN | STDOUT | STDERR => Some(false),
            _ => {
                let stream_info_ref: Ref<Vec<Stream>> = system.stream_info.borrow();
                if index >= stream_info_ref.len() {
                    None
                } else {
                    match stream_info_ref.get(index).unwrap() {
                        Stream::String(_) => Some(true),
                        _ => Some(false),
                    }
                }
            }
        }
    }

    fn flush(system: &System, index: usize) -> Option<()> {
        match index {
            STDOUT => {
                std::io::stdout().flush().unwrap();
            }
            STDERR => {
                std::io::stderr().flush().unwrap();
            }
            _ => {
                let stream_info_ref: Ref<Vec<Stream>> = system.stream_info.borrow();
                if index >= stream_info_ref.len() {
                    return None;
                }
            }
        };

        Some(())
    }

    fn close(system: &System, index: usize) -> Option<()> {
        match index {
            STDIN | STDOUT | STDERR => (),
            _ => {
                let stream_info_ref: Ref<Vec<Stream>> = system.stream_info.borrow();
                if index >= stream_info_ref.len() {
                    return None;
                } else {
                    match stream_info_ref.get(index).unwrap() {
                        Stream::File(file) => {
                            let file_ref: Ref<fs::File> = file.borrow();
                            std::mem::drop(file_ref)
                        }
                        Stream::String(_) => (),
                    }
                }
            }
        };

        Some(())
    }

    fn open_file(system: &System, path: &str, is_input: bool) -> exception::Result<usize> {
        let stream = if is_input {
            StreamBuilder::new().file(path.to_string()).input().build()
        } else {
            StreamBuilder::new().file(path.to_string()).output().build()
        };

        let file = match stream {
            Some(Stream::File(f)) => f,
            _ => {
                return Err(Exception {
                    object: Tag::nil(),
                    condition: Condition::Open,
                    source: "system::open".to_string(),
                })
            }
        };

        let mut stream_info_ref: RefMut<Vec<Stream>> = system.stream_info.borrow_mut();
        let index = stream_info_ref.len();

        stream_info_ref.push(Stream::File(file));

        Ok(index)
    }

    fn open_string(system: &System, contents: &str, is_input: bool) -> exception::Result<usize> {
        let stream = if is_input {
            StreamBuilder::new()
                .string(contents.to_string())
                .input()
                .build()
        } else {
            StreamBuilder::new()
                .string(contents.to_string())
                .output()
                .build()
        };

        let string = match stream {
            Some(Stream::String(str)) => str,
            _ => {
                return Err(Exception {
                    object: Tag::nil(),
                    condition: Condition::Open,
                    source: "system::open".to_string(),
                })
            }
        };

        let mut stream_info_ref: RefMut<Vec<Stream>> = system.stream_info.borrow_mut();
        let index = stream_info_ref.len();

        stream_info_ref.push(Stream::String(string));

        Ok(index)
    }

    fn get_string(system: &System, index: usize) -> Option<String> {
        match index {
            STDIN | STDOUT | STDERR => None,
            _ => {
                let stream_info_ref: Ref<Vec<Stream>> = system.stream_info.borrow();
                if index >= stream_info_ref.len() {
                    return None;
                }

                match stream_info_ref.get(index).unwrap() {
                    Stream::File(_) => None,
                    Stream::String(string) => {
                        let mut string_ref: RefMut<VecDeque<u8>> = string.borrow_mut();
                        let string_vec: Vec<u8> = string_ref.iter().cloned().collect();

                        string_ref.clear();
                        Some(str::from_utf8(&string_vec).unwrap().to_owned())
                    }
                }
            }
        }
    }

    fn read_byte(system: &System, stream_id: usize) -> exception::Result<Option<u8>> {
        let stream_info_ref: Ref<Vec<Stream>> = system.stream_info.borrow();
        let mut buf = [0; 1];

        match stream_id {
            STDIN => match std::io::stdin().read(&mut buf) {
                Ok(nread) => {
                    if nread == 0 {
                        Ok(None)
                    } else {
                        Ok(Some(buf[0]))
                    }
                }
                Err(_) => Err(Exception {
                    object: Tag::nil(),
                    condition: Condition::Read,
                    source: "system::read_byte".to_string(),
                }),
            },
            _ if stream_id < stream_info_ref.len() => match stream_info_ref.get(stream_id).unwrap()
            {
                Stream::File(file) => {
                    let mut file_ref: RefMut<fs::File> = file.borrow_mut();
                    match file_ref.read(&mut buf) {
                        Ok(nread) => {
                            if nread == 0 {
                                Ok(None)
                            } else {
                                Ok(Some(buf[0]))
                            }
                        }
                        Err(_) => Err(Exception {
                            object: Tag::nil(),
                            condition: Condition::Read,
                            source: "system::read_byte".to_string(),
                        }),
                    }
                }
                Stream::String(string) => {
                    let mut string_ref: RefMut<VecDeque<u8>> = string.borrow_mut();

                    if string_ref.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(string_ref.pop_front().unwrap()))
                    }
                }
            },
            _ => panic!(),
        }
    }

    fn write_byte(system: &System, stream_id: usize, byte: u8) -> exception::Result<Option<()>> {
        let stream_info_ref: Ref<Vec<Stream>> = system.stream_info.borrow();
        let buf = [byte; 1];

        match stream_id {
            STDOUT => match std::io::stdout().write(&buf) {
                Ok(_) => Ok(None),
                Err(_) => Err(Exception {
                    object: Tag::nil(),
                    condition: Condition::Write,
                    source: "system::write_byte".to_string(),
                }),
            },
            STDERR => match std::io::stderr().write(&buf) {
                Ok(_) => Ok(None),
                Err(_) => Err(Exception {
                    object: Tag::nil(),
                    condition: Condition::Write,
                    source: "system::write_byte".to_string(),
                }),
            },
            _ if stream_id < stream_info_ref.len() => {
                match stream_info_ref.get(stream_id).unwrap() {
                    Stream::File(file) => {
                        let mut file_ref: RefMut<fs::File> = file.borrow_mut();
                        match file_ref.write(&buf) {
                            Ok(_) => Ok(None),
                            Err(_) => Err(Exception {
                                object: Tag::nil(),
                                condition: Condition::Write,
                                source: "system::write_byte".to_string(),
                            }),
                        }
                    }
                    Stream::String(string) => {
                        let mut string_ref: RefMut<VecDeque<u8>> = string.borrow_mut();

                        string_ref.push_back(buf[0]);
                        Ok(Some(()))
                    }
                }
            }
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn stream() {
        assert_eq!(true, true)
    }
}
