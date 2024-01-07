//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! The Thorn Mu Virtual Machine
//!
//! The thorn-mu virtual machine is the implementation surface for the [`thorn programming environment`].
//!
//! As much as is practible, thorn-mu's functions and data types resemble Common Lisp in preference to
//! Scheme/Clojure in order to be immediately familiar to the traditional LISP programmer.
//!
//! thorn-mu is an immutable, lexically scoped LISP-1 kernel meant as a porting layer for an ascending
//! tower of LISP languages. While it is possible to do some useful application work directly in the
//! thorn-mu language, thorn-mu defers niceties like macros, closures, and rest functions to a compiler
//! layered on it. See [`thorn programming environment`] for details.
//!
//! thorn-mu characteristics:
//! - 100% mostly-safe Rust
//! - 64 bit immutable tagged objects
//! - garbage collected heap
//! - lambda compiler
//! - minimal external dependencies
//! - multiple independent execution environments
//! - s-expression reader/printer
//! - symbol namespaces
//!
//! thorn-mu data types:
//!    56 bit fixnums (immediate)
//!    Lisp-1 symbols
//!    character, string, and byte streams
//!    characters (ASCII immediate)
//!    conses
//!    fixed arity functions
//!    lambdas with lexical variables
//!    general and specialized vectors
//!    keywords (seven character immediate)
//!    single/32 bit IEEE float (immediate)
//!    structs
//!    symbol namespaces
//!
//! thorn-mu documentation:
//!    see doc/refcards and doc/rustdoc
//!
//! [`thorn programming environment`]: <https://github.com/Software-Knife-and-Tool/thorn>
//!
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate modular_bitfield;

mod allocators;
mod async_;
mod core;
mod system;
mod types;

use {
    crate::core::{
        compile::Compiler,
        config::Config,
        exception,
        mu::{self, Core},
        stream::{self, Core as _},
    },
    std::fs,
    types::{
        stream::{Core as _, Stream},
        streambuilder::StreamBuilder,
    },
};

/// The thorn-mu API
///
/// The thorn-mu API exposes these types:
/// - Tag, tagged data representation
/// - Result, specialized result for API functions that can fail
/// - Exception, exception state
/// - Condition, enumeration of possible exceptional conditions
/// - Mu, environment and API namespace
/// - System, an optional interface to Mu

/// the tagged data representation
pub type Tag = core::types::Tag;
/// the API function Result
pub type Result = core::exception::Result<Tag>;
/// the condition enumeration
pub type Condition = core::exception::Condition;
/// the Exception representation
pub type Exception = core::exception::Exception;

/// the Mu struct abstracts the core library struct
pub struct Mu(core::mu::Mu);

impl Mu {
    /// current version
    pub const VERSION: &'static str = core::mu::Mu::VERSION;

    /// config
    pub fn config(config_string: &String) -> Option<Config> {
        core::mu::Mu::config(config_string.to_string())
    }

    /// constructor
    pub fn new(config: &Config) -> Self {
        Mu(core::mu::Mu::new(config))
    }

    /// apply a function to a list of arguments
    pub fn apply(&self, func: Tag, args: Tag) -> exception::Result<Tag> {
        self.0.apply(func, args)
    }

    /// test tagged s-expressions for strict equality
    pub fn eq(&self, tag: Tag, tag1: Tag) -> bool {
        tag.eq_(&tag1)
    }

    /// evaluate a tagged s-expression
    pub fn eval(&self, expr: Tag) -> exception::Result<Tag> {
        self.0.eval(expr)
    }

    /// compile a tagged s-expression
    pub fn compile(&self, expr: Tag) -> exception::Result<Tag> {
        <mu::Mu as Compiler>::compile(&self.0, expr)
    }

    /// read a tagged s-expression from a mu stream
    pub fn read(&self, stream: Tag, eofp: bool, eof_value: Tag) -> exception::Result<Tag> {
        <mu::Mu as stream::Core>::read(&self.0, stream, eofp, eof_value, false)
    }

    /// convert a rust String to a tagged s-expression
    pub fn read_string(&self, string: String) -> exception::Result<Tag> {
        match StreamBuilder::new().string(string).input().build(&self.0) {
            Ok(stream) => <mu::Mu as stream::Core>::read(
                &self.0,
                stream.evict(&self.0),
                true,
                Tag::nil(),
                false,
            ),
            Err(e) => Err(e),
        }
    }

    /// write a human-readable representation of an s-expression to a mu stream
    pub fn write(&self, expr: Tag, escape: bool, stream: Tag) -> exception::Result<()> {
        self.0.write(expr, escape, stream)
    }

    /// get a rust String from a string output stream
    pub fn get_string(&self, stream: Tag) -> exception::Result<String> {
        Stream::get_string(&self.0, stream)
    }

    /// write a rust String to a mu stream
    pub fn write_string(&self, str: &str, stream: Tag) -> exception::Result<()> {
        self.0.write_string(str, stream)
    }

    /// deserialize a tag
    pub fn from_u64(&self, tag: u64) -> Tag {
        Tag::from_u64(tag)
    }

    /// serialize a tag
    pub fn as_u64(&self, tag: Tag) -> u64 {
        tag.as_u64()
    }

    /// return the standard-input mu stream
    pub fn std_in(&self) -> Tag {
        self.0.stdin
    }

    /// return the standard-output mu stream
    pub fn std_out(&self) -> Tag {
        self.0.stdout
    }

    /// return the error-output mu stream
    pub fn err_out(&self) -> Tag {
        self.0.errout
    }
}

/// System is a convenience layer over Mu
pub struct System {
    mu: Mu,
    sys_stream: Tag,
}

impl System {
    #[allow(clippy::new_without_default)]
    pub fn new(config: &Config) -> Self {
        let mu = Mu::new(config);

        let sys_stream = mu
            .eval(
                mu.compile(
                    mu.read_string("(mu:open :string :output \"\")".to_string())
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();

        System { mu, sys_stream }
    }

    pub fn config(conf: &String) -> Option<Config> {
        Mu::config(&conf.to_string())
    }

    pub fn mu(&self) -> &Mu {
        &self.mu
    }

    pub fn eval(&self, expr: &String) -> Result {
        match self.mu.read_string(expr.to_string()) {
            Ok(expr) => match self.mu.compile(expr) {
                Ok(expr) => self.mu.eval(expr),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    pub fn error(&self, ex: Exception) -> String {
        format!(
            "error: condition {:?} on {} raised by {}",
            ex.condition,
            self.write(ex.object, true),
            self.write(ex.source, true),
        )
    }

    pub fn read(&self, string: String) -> Result {
        match StreamBuilder::new()
            .string(string)
            .input()
            .build(&self.mu.0)
        {
            Ok(stream) => <mu::Mu as stream::Core>::read(
                &self.mu.0,
                stream.evict(&self.mu.0),
                true,
                Tag::nil(),
                false,
            ),
            Err(e) => Err(e),
        }
    }

    pub fn write(&self, expr: Tag, escape: bool) -> String {
        self.mu.write(expr, escape, self.sys_stream).unwrap();
        self.mu.get_string(self.sys_stream).unwrap()
    }

    pub fn load(&self, file_path: &String) -> Result {
        if fs::metadata(file_path).is_ok() {
            let load_form = format!("(mu:open :file :input \"{}\")", file_path);
            let istream = self
                .mu
                .eval(self.mu.read_string(load_form).unwrap())
                .unwrap();
            let eof_value = self.mu.read_string(":eof".to_string()).unwrap(); // need make_symbol here

            #[allow(clippy::while_let_loop)]
            loop {
                match self.mu.read(istream, true, eof_value) {
                    Ok(form) => {
                        if self.mu.eq(form, eof_value) {
                            return Ok(istream);
                        }
                        match self.mu.compile(form) {
                            Ok(form) => match self.mu.eval(form) {
                                Ok(_) => (),
                                Err(e) => return Err(e),
                            },
                            Err(e) => return Err(e),
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
        } else {
            Err(Exception::new(
                Condition::Open,
                "sys:lf",
                self.mu.read_string(format!("\"{}\"", file_path)).unwrap(),
            ))
        }
    }
}
