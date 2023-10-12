//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu functions
use crate::{
    core::{
        backquote::MuFunction as _,
        compile::Compiler,
        exception::{self, Condition, Exception, MuFunction as _},
        frame::{Frame, MuFunction as _},
        indirect::MuFunction as _,
        mu::{Core as _, Mu},
        namespace::{Core as NSCore, MuFunction as _},
        types::{MuFunction as _, Tag, Type},
    },
    system::sys::System,
    types::{
        char::{Char, Core as _},
        cons::{Cons, ConsIter, Core as _, MuFunction as _},
        fixnum::{Core as _, Fixnum, MuFunction as _},
        float::{Core as _, Float, MuFunction as _},
        function::{Core as _, Function},
        stream::{Core as _, MuFunction as _, Stream},
        struct_::{Core as _, MuFunction as _, Struct},
        symbol::{Core as _, MuFunction as _, Symbol},
        vecimage::{TypedVec, VecType},
        vector::{Core as _, MuFunction as _, Vector},
    },
};

use crate::core::async_context::MuFunction as _;

// native functions
pub type LibMuFunction = fn(&Mu, &mut Frame) -> exception::Result<()>;

// mu function dispatch table
lazy_static! {
    static ref SYMBOLMAP: Vec<(&'static str, u16, LibMuFunction)> = vec![
        // types
        ("eq", 2, Tag::mu_eq),
        ("type-of", 1, Tag::mu_typeof),
        // conses and lists
        ("car", 1, Cons::mu_car),
        ("cdr", 1, Cons::mu_cdr),
        ("cons", 2, Cons::mu_cons),
        ("length", 1, Cons::mu_length),
        ("nth", 2, Cons::mu_nth),
        ("nthcdr", 2, Cons::mu_nthcdr),
        // async
        ("await", 1, Mu::mu_await),
        ("abort", 1, Mu::mu_abort),
        // mu
        ("apply", 2, Mu::mu_apply),
        ("compile", 1, Mu::mu_compile),
        ("eval", 1, Mu::mu_eval),
        ("exit", 1, Mu::mu_exit),
        ("fix", 2, Mu::mu_fix),
        ("hp-info", 0, Mu::mu_hp_info),
        ("size-of", 1, Mu::mu_size_of),
        ("view", 1, Mu::mu_view),
        ("repr", 2, Mu::mu_repr),
        ("%append", 2, Mu::append_),
        // time
        ("real-tm", 0, Mu::mu_real_time),
        ("run-us", 0, Mu::mu_run_time),
        // exceptions
        ("with-ex", 2, Exception::mu_with_ex),
        ("raise", 2, Exception::mu_raise),
        // frames
        ("frames", 0, Frame::mu_frames),
        ("fr-pop", 1, Frame::mu_fr_pop),
        ("fr-push", 1, Frame::mu_fr_push),
        ("fr-ref", 2, Frame::mu_fr_ref),
        // fixnums
        ("fx-add", 2, Fixnum::mu_fxadd),
        ("fx-sub", 2, Fixnum::mu_fxsub),
        ("fx-lt", 2, Fixnum::mu_fxlt),
        ("fx-mul", 2, Fixnum::mu_fxmul),
        ("fx-div", 2, Fixnum::mu_fxdiv),
        ("logand", 2, Fixnum::mu_fxand),
        ("logor", 2, Fixnum::mu_fxor),
        // floats
        ("fl-add", 2, Float::mu_fladd),
        ("fl-sub", 2, Float::mu_flsub),
        ("fl-lt", 2, Float::mu_fllt),
        ("fl-mul", 2, Float::mu_flmul),
        ("fl-div", 2, Float::mu_fldiv),
        // namespaces
        ("untern", 2, Mu::mu_untern),
        ("intern", 3, Mu::mu_intern),
        ("make-ns", 1, Mu::mu_make_ns),
        ("ns-syms", 1, Mu::mu_ns_symbols),
        ("ns-find", 2, Mu::mu_ns_find),
        // read/write
        ("read", 3, Stream::mu_read),
        ("write", 3, Stream::mu_write),
        // symbols
        ("boundp", 1, Symbol::mu_boundp),
        ("keyword", 1, Symbol::mu_keyword),
        ("make-sy", 1, Symbol::mu_symbol),
        ("sy-name", 1, Symbol::mu_name),
        ("sy-ns", 1, Symbol::mu_ns),
        ("sy-val", 1, Symbol::mu_value),
        // simple vectors
        ("make-sv", 2, Vector::mu_make_vector),
        ("sv-len", 1, Vector::mu_length),
        ("sv-ref", 2, Vector::mu_svref),
        ("sv-type", 1, Vector::mu_type),
        // structs
        ("make-st", 2, Struct::mu_make_struct),
        ("st-type", 1, Struct::mu_struct_type),
        ("st-vec", 1, Struct::mu_struct_vector),
        // streams
        ("close", 1, Stream::mu_close),
        ("eof", 1, Stream::mu_eof),
        ("flush", 1, Stream::mu_flush),
        ("get-str", 1, Stream::mu_get_string),
        ("open", 3, Stream::mu_open),
        ("openp", 1, Stream::mu_openp),
        ("rd-byte", 3, Stream::mu_read_byte),
        ("rd-char", 3, Stream::mu_read_char),
        ("un-char", 2, Stream::mu_unread_char),
        ("wr-byte", 2, Stream::mu_write_byte),
        ("wr-char", 2, Stream::mu_write_char),
        ("%append", 2, Mu::append_),
        ("%if", 3, Mu::if_),
    ];
}

pub trait Core {
    fn install_lib_functions(_: &Mu) -> Vec<LibMuFunction>;
}

impl Core for Mu {
    fn install_lib_functions<'a>(mu: &Mu) -> Vec<LibMuFunction> {
        let mut funcv = Vec::new();

        for (id, fnmap) in SYMBOLMAP.iter().enumerate() {
            let (name, nreqs, libfn) = fnmap;

            let func = Function::new(
                Fixnum::as_tag(*nreqs as i64),
                Fixnum::as_tag(match id.try_into().unwrap() {
                    Some(n) => n as i64,
                    None => panic!(),
                }),
                Symbol::new(mu, Tag::nil(), name, Tag::nil()).evict(mu),
            )
            .evict(mu);

            funcv.push(*libfn);

            <Mu as NSCore>::intern_symbol(mu, mu.mu_ns, name.to_string(), func);
        }

        funcv
    }
}

pub trait MuFunction {
    fn mu_apply(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_compile(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_eval(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_exit(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fix(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn if_(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_real_time(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_repr(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_run_time(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_view(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_size_of(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_write(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_real_time(_: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match System::real_time() {
            Ok(us) => Fixnum::as_tag(us as i64),
            Err(_) => return Err(Exception::new(Condition::Error, "real-us", Tag::nil())),
        };

        Ok(())
    }

    fn mu_run_time(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let time = mu.start_time.elapsed();
        let usec = time.as_micros();

        fp.value = Fixnum::as_tag(usec.try_into().unwrap());

        Ok(())
    }

    fn mu_compile(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match <Mu as Compiler>::compile(mu, fp.argv[0]) {
            Ok(tag) => tag,
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_eval(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match mu.eval(fp.argv[0]) {
            Ok(tag) => tag,
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_apply(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let func = fp.argv[0];
        let args = fp.argv[1];

        fp.value = match Tag::type_of(func) {
            Type::Function => match Tag::type_of(args) {
                Type::Null | Type::Cons => {
                    let value = Tag::nil();
                    let mut argv = Vec::new();

                    for cons in ConsIter::new(mu, args) {
                        argv.push(Cons::car(mu, cons))
                    }

                    match (Frame { func, argv, value }).apply(mu, func) {
                        Ok(value) => value,
                        Err(e) => return Err(e),
                    }
                }
                _ => return Err(Exception::new(Condition::Type, "apply", args)),
            },
            _ => return Err(Exception::new(Condition::Type, "apply", func)),
        };

        Ok(())
    }

    fn mu_write(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let form = fp.argv[0];
        let escape = fp.argv[1];
        let stream = fp.argv[2];

        fp.value = form;

        match Tag::type_of(stream) {
            Type::Stream => match mu.write(form, !escape.null_(), stream) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            _ => Err(Exception::new(Condition::Type, "write", stream)),
        }
    }

    fn if_(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let test = fp.argv[0];
        let true_fn = fp.argv[1];
        let false_fn = fp.argv[2];

        fp.value = match Tag::type_of(true_fn) {
            Type::Function => match Tag::type_of(false_fn) {
                Type::Function => {
                    match mu.apply(if test.null_() { false_fn } else { true_fn }, Tag::nil()) {
                        Ok(tag) => tag,
                        Err(e) => return Err(e),
                    }
                }
                _ => return Err(Exception::new(Condition::Type, "::if", false_fn)),
            },
            _ => return Err(Exception::new(Condition::Type, "::if", true_fn)),
        };

        Ok(())
    }

    fn mu_exit(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let rc = fp.argv[0];

        match Tag::type_of(rc) {
            Type::Fixnum => std::process::exit(Fixnum::as_i64(mu, rc) as i32),
            _ => Err(Exception::new(Condition::Type, "exit", rc)),
        }
    }

    fn mu_repr(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let conv = fp.argv[0];
        let arg = fp.argv[1];

        // if conv is (), convert arg tag bits to a byte vector
        // if not, convert arg byte vector to a tag

        fp.value = match Tag::null_(&conv) {
            true => {
                let slice = arg.as_slice();

                TypedVec::<Vec<u8>> {
                    vec: slice.to_vec(),
                }
                .vec
                .to_vector()
                .evict(mu)
            }
            false => match Tag::type_of(arg) {
                Type::Vector
                    if Vector::type_of(mu, arg) == Type::Byte && Vector::length(mu, arg) == 8 =>
                {
                    let mut u64_: u64 = 0;

                    for index in (0..8).rev() {
                        u64_ <<= 8;
                        u64_ |= match Vector::ref_(mu, arg, index as usize) {
                            Some(byte) => Fixnum::as_i64(mu, byte) as u64,
                            None => panic!(),
                        }
                    }

                    Tag::from_u64(u64_)
                }
                _ => return Err(Exception::new(Condition::Type, "repr", arg)),
            },
        };

        Ok(())
    }

    fn mu_size_of(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let tag = fp.argv[0];

        fp.value = match mu.size_of(tag) {
            Ok(size) => Fixnum::as_tag(size as i64),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn mu_view(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let tag = fp.argv[0];

        fp.value = match Tag::type_of(tag) {
            Type::Char => Char::view(mu, tag),
            Type::Cons => Cons::view(mu, tag),
            Type::Fixnum => Fixnum::view(mu, tag),
            Type::Float => Float::view(mu, tag),
            Type::Function => Function::view(mu, tag),
            Type::Null | Type::Symbol | Type::Keyword => Symbol::view(mu, tag),
            Type::Stream => Stream::view(mu, tag),
            Type::Struct => Struct::view(mu, tag),
            Type::Vector => Vector::view(mu, tag),
            _ => return Err(Exception::new(Condition::Type, "view", tag)),
        };

        Ok(())
    }

    fn mu_fix(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let func = fp.argv[0];

        fp.value = fp.argv[1];

        match Tag::type_of(func) {
            Type::Function => {
                loop {
                    let value = Tag::nil();
                    let argv = vec![fp.value];
                    let result = Frame { func, argv, value }.apply(mu, func);

                    fp.value = match result {
                        Ok(value) => {
                            if value.eq_(fp.value) {
                                break;
                            }

                            value
                        }
                        Err(e) => return Err(e),
                    };
                }

                Ok(())
            }
            _ => Err(Exception::new(Condition::Type, "fix", func)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mu_functions() {
        assert_eq!(2 + 2, 4);
    }
}
