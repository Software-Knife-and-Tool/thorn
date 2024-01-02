//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu functions
use crate::{
    async_::context::MuFunction as _,
    core::{
        backquote::MuFunction as _,
        compile::MuFunction as _,
        dynamic::MuFunction as _,
        exception::{self, Condition, Exception, MuFunction as _},
        frame::{Frame, MuFunction as _},
        heap::MuFunction as _,
        mu::{Core as _, Mu},
        namespace::{Core as NSCore, MuFunction as _},
        stream::MuFunction as _,
        system::MuFunction as _,
        types::{MuFunction as _, Tag, Type},
    },
    types::{
        cons::{Cons, ConsIter, MuFunction as _},
        fixnum::{Fixnum, MuFunction as _},
        float::{Float, MuFunction as _},
        function::Function,
        map::MuFunction as _,
        stream::Stream,
        streams::MuFunction as _,
        struct_::{MuFunction as _, Struct},
        symbol::{MuFunction as _, Symbol},
        vector::{MuFunction as _, Vector},
    },
};

#[cfg(feature = "qquote")]
use crate::core::qquote::{MuFunction as _, QqReader};

//
// native functions
//
pub type LibMuFunction = fn(&Mu, &mut Frame) -> exception::Result<()>;

// mu function dispatch table
lazy_static! {
    static ref MU_SYMBOLS: Vec<(&'static str, u16, LibMuFunction)> = vec![
        // types
        ("eq", 2, Tag::mu_eq),
        ("type-of", 1, Tag::mu_typeof),
        ("repr", 2, Tag::mu_repr),
        ("view", 1, Tag::mu_view),
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
        // maps
        ("map", 1, Mu::mu_make_map),
        ("mp-has", 2, Mu::mu_map_has),
        ("mp-list", 1, Mu::mu_map_items),
        ("mp-ref", 2, Mu::mu_map_ref),
        ("mp-size", 1, Mu::mu_map_size),
        // heap
        ("gc", 0, Mu::mu_gc),
        ("hp-info", 0, Mu::mu_hp_info),
        ("hp-stat", 0, Mu::mu_hp_stat),
        ("hp-size", 1, Mu::mu_hp_size),
        // mu
        ("apply", 2, Mu::mu_apply),
        ("compile", 1, Mu::mu_compile),
        ("eval", 1, Mu::mu_eval),
        ("frames", 0, Mu::mu_frames),
        ("fix", 2, Mu::mu_fix),
        #[cfg(feature = "qquote")]
        ("%qquote", 1, QqReader::mu_qquote),
        ("%append", 2, Mu::append_),
        // exceptions
        ("with-ex", 2, Exception::mu_with_ex),
        ("raise", 2, Exception::mu_raise),
        // frames
        ("fr-pop", 1, Frame::mu_fr_pop),
        ("fr-push", 1, Frame::mu_fr_push),
        ("fr-ref", 2, Frame::mu_fr_ref),
        // fixnums
        ("fx-ash", 2, Fixnum::mu_fxash),
        ("fx-add", 2, Fixnum::mu_fxadd),
        ("fx-sub", 2, Fixnum::mu_fxsub),
        ("fx-lt", 2, Fixnum::mu_fxlt),
        ("fx-mul", 2, Fixnum::mu_fxmul),
        ("fx-div", 2, Fixnum::mu_fxdiv),
        ("logand", 2, Fixnum::mu_logand),
        ("logor", 2, Fixnum::mu_logor),
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
        ("ns-syms", 2, Mu::mu_ns_symbols),
        ("ns-find", 2, Mu::mu_ns_find),
        ("ns-map", 0, Mu::mu_ns_map),
        // read/write
        ("read", 3, Mu::mu_read),
        ("write", 3, Mu::mu_write),
        // symbols
        ("boundp", 1, Symbol::mu_boundp),
        ("keyword", 1, Symbol::mu_keyword),
        ("symbol", 1, Symbol::mu_symbol),
        ("sy-name", 1, Symbol::mu_name),
        ("sy-ns", 1, Symbol::mu_ns),
        ("sy-val", 1, Symbol::mu_value),
        // simple vectors
        ("vector", 2, Vector::mu_make_vector),
        ("sv-len", 1, Vector::mu_length),
        ("sv-ref", 2, Vector::mu_svref),
        ("sv-type", 1, Vector::mu_type),
        // structs
        ("struct", 2, Struct::mu_make_struct),
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

    static ref SYS_SYMBOLS: Vec<(&'static str, u16, LibMuFunction)> = vec![
        // system
        ("exit", 1, Mu::sys_exit),
        ("real-tm", 0, Mu::sys_real_time),
        ("run-us", 0, Mu::sys_run_time),
    ];
}

impl Mu {
    pub fn install_lib_functions(mu: &Mu) -> Vec<LibMuFunction> {
        let mut funcv = Vec::new();

        for (id, fnmap) in MU_SYMBOLS.iter().enumerate() {
            let (name, nreqs, libfn) = fnmap;

            let func = Function::new(
                Fixnum::as_tag(*nreqs as i64),
                Fixnum::as_tag(match id.try_into().unwrap() {
                    Some(n) => n as i64,
                    None => panic!(),
                }),
            )
            .evict(mu);

            funcv.push(*libfn);

            <Mu as NSCore>::intern_symbol(mu, mu.mu_ns, name.to_string(), func);
        }

        let fn_tab_offset = funcv.len();
        for (id, fnmap) in SYS_SYMBOLS.iter().enumerate() {
            let (name, nreqs, libfn) = fnmap;

            let func = Function::new(
                Fixnum::as_tag(*nreqs as i64),
                Fixnum::as_tag(match (id + fn_tab_offset).try_into().unwrap() {
                    Some(n) => n as i64,
                    None => panic!(),
                }),
            )
            .evict(mu);

            funcv.push(*libfn);

            <Mu as NSCore>::intern_symbol(mu, mu.sys_ns, name.to_string(), func);
        }

        funcv
    }
}

pub trait Core {
    fn fp_argv_check(&self, _: &str, _: &[Type], _: &Frame) -> exception::Result<()>;
}

impl Core for Mu {
    fn fp_argv_check(&self, fn_name: &str, types: &[Type], fp: &Frame) -> exception::Result<()> {
        for (index, arg_type) in types.iter().enumerate() {
            let fp_arg_type = Tag::type_of(fp.argv[index]);
            let fp_arg = fp.argv[index];

            match *arg_type {
                Type::Byte => match fp_arg_type {
                    Type::Fixnum => {
                        let n = Fixnum::as_i64(fp_arg);

                        if !(0..=255).contains(&n) {
                            return Err(Exception::new(Condition::Type, fn_name, fp_arg));
                        }
                    }
                    _ => return Err(Exception::new(Condition::Type, fn_name, fp_arg)),
                },
                Type::List => match fp_arg_type {
                    Type::Cons | Type::Null => (),
                    _ => return Err(Exception::new(Condition::Type, fn_name, fp_arg)),
                },
                Type::String => match fp_arg_type {
                    Type::Vector => {
                        if Vector::type_of(self, fp.argv[index]) != Type::Char {
                            return Err(Exception::new(Condition::Type, fn_name, fp_arg));
                        }
                    }
                    _ => return Err(Exception::new(Condition::Type, fn_name, fp_arg)),
                },
                Type::T => (),
                _ => {
                    if fp_arg_type != *arg_type {
                        return Err(Exception::new(Condition::Type, fn_name, fp_arg));
                    }
                }
            }
        }

        Ok(())
    }
}

pub trait MuFunction {
    fn mu_apply(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_eval(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fix(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
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

        fp.value = match mu.fp_argv_check("apply", &[Type::Function, Type::List], fp) {
            Ok(_) => {
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
            Err(e) => return Err(e),
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
