//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu functions
// use cpu_time::ProcessTime;
// use std::time::Duration;
use crate::{
    core::{
        async_::{Async, MuFunction as _},
        backquote::MuFunction as _,
        compile::Compiler,
        exception::{self, Condition, Exception, MuFunction as _},
        frame::{Frame, MuFunction as _},
        indirect::MuFunction as _,
        mu::{Core as _, Mu},
        types::{MuFunction as _, Tag, Type},
    },
    system::sys::System,
    types::{
        char::{Char, Core as _},
        cons::{Cons, ConsIter, Core as _, MuFunction as _},
        fixnum::{Core as _, Fixnum, MuFunction as _},
        float::{Core as _, Float, MuFunction as _},
        function::{Core as _, Function},
        namespace::{Core as _, MuFunction as _, Namespace, Scope},
        stream::{Core as _, MuFunction as _, Stream},
        struct_::{Core as _, MuFunction as _, Struct},
        symbol::{Core as _, MuFunction as _, Symbol},
        vector::{Core as _, MuFunction as _, Vector},
    },
};

// native functions
pub type LibFunction = fn(&Mu, &mut Frame) -> exception::Result<()>;
pub type InternalFunction = (String, Tag);

// mu function dispatch table
lazy_static! {
    static ref SYMBOLMAP: Vec<(&'static str, Option<Scope>, u16, LibFunction)> = vec![
        // types
        ("eq", Some(Scope::Extern), 2, Tag::mu_eq),
        ("type-of", Some(Scope::Extern), 1, Tag::mu_typeof),
        // conses and lists
        ("car", Some(Scope::Extern), 1, Cons::mu_car),
        ("cdr", Some(Scope::Extern), 1, Cons::mu_cdr),
        ("cons", Some(Scope::Extern), 2, Cons::mu_cons),
        ("length", Some(Scope::Extern), 1, Cons::mu_length),
        ("nth", Some(Scope::Extern), 2, Cons::mu_nth),
        ("nthcdr", Some(Scope::Extern), 2, Cons::mu_nthcdr),
        // async
        ("async", Some(Scope::Extern), 2, Async::mu_async),
        ("await", Some(Scope::Extern), 2, Async::mu_await),
        // mu
        ("apply", Some(Scope::Extern), 2, Mu::mu_apply),
        ("compile", Some(Scope::Extern), 1, Mu::mu_compile),
        ("eval", Some(Scope::Extern), 1, Mu::mu_eval),
        ("exit", Some(Scope::Intern), 1, Mu::mu_exit),
        ("fix", Some(Scope::Extern), 2, Mu::mu_fix),
        ("hp-info", Some(Scope::Extern), 0, Mu::mu_hp_info),
        ("view", Some(Scope::Extern), 1, Mu::mu_view),
        // time
        ("real-tm", Some(Scope::Extern), 0, Mu::mu_real_time),
        ("run-us", Some(Scope::Extern), 0, Mu::mu_run_time),
        // exceptions
        ("with-ex", Some(Scope::Extern), 2, Exception::mu_with_ex),
        ("raise", Some(Scope::Extern), 2, Exception::mu_raise),
        // frames
        ("frames", Some(Scope::Intern), 0, Frame::mu_frames),
        ("fr-get", Some(Scope::Extern), 1, Frame::mu_fr_get),
        ("fr-pop", Some(Scope::Extern), 1, Frame::mu_fr_pop),
        ("fr-push", Some(Scope::Extern), 1, Frame::mu_fr_push),
        ("fr-ref", Some(Scope::Intern), 2, Frame::mu_fr_ref),
        // fixnums
        ("fx-add", Some(Scope::Extern), 2, Fixnum::mu_fxadd),
        ("fx-sub", Some(Scope::Extern), 2, Fixnum::mu_fxsub),
        ("fx-lt", Some(Scope::Extern), 2, Fixnum::mu_fxlt),
        ("fx-mul", Some(Scope::Extern), 2, Fixnum::mu_fxmul),
        ("fx-div", Some(Scope::Extern), 2, Fixnum::mu_fxdiv),
        ("logand", Some(Scope::Extern), 2, Fixnum::mu_fxand),
        ("logor", Some(Scope::Extern), 2, Fixnum::mu_fxor),
        // floats
        ("fl-add", Some(Scope::Extern), 2, Float::mu_fladd),
        ("fl-sub", Some(Scope::Extern), 2, Float::mu_flsub),
        ("fl-lt", Some(Scope::Extern), 2, Float::mu_fllt),
        ("fl-mul", Some(Scope::Extern), 2, Float::mu_flmul),
        ("fl-div", Some(Scope::Extern), 2, Float::mu_fldiv),
        // namespaces
        ("untern", Some(Scope::Extern), 3, Namespace::mu_untern),
        ("intern", Some(Scope::Extern), 4, Namespace::mu_intern),
        ("make-ns", Some(Scope::Extern), 2, Namespace::mu_make_ns),
        ("map-ns", Some(Scope::Extern), 1, Namespace::mu_map_ns),
        ("ns-ext", Some(Scope::Extern), 1, Namespace::mu_ns_externs),
        ("ns-imp", Some(Scope::Extern), 1, Namespace::mu_ns_import),
        ("ns-int", Some(Scope::Extern), 1, Namespace::mu_ns_interns),
        ("ns-find", Some(Scope::Extern), 3, Namespace::mu_ns_find),
        ("ns-name", Some(Scope::Extern), 1, Namespace::mu_ns_name),
        // read/write
        ("read", Some(Scope::Extern), 3, Stream::mu_read),
        ("write", Some(Scope::Extern), 3, Stream::mu_write),
        // symbols
        ("boundp", Some(Scope::Extern), 1, Symbol::mu_boundp),
        ("keyword", Some(Scope::Extern), 1, Symbol::mu_keyword),
        ("make-sy", Some(Scope::Extern), 1, Symbol::mu_symbol),
        ("sy-name", Some(Scope::Extern), 1, Symbol::mu_name),
        ("sy-ns", Some(Scope::Extern), 1, Symbol::mu_ns),
        ("sy-val", Some(Scope::Extern), 1, Symbol::mu_value),
        // simple vectors
        ("make-sv", Some(Scope::Extern), 2, Vector::mu_make_vector),
        ("sv-len", Some(Scope::Extern), 1, Vector::mu_length),
        ("sv-ref", Some(Scope::Extern), 2, Vector::mu_svref),
        ("sv-type", Some(Scope::Extern), 1, Vector::mu_type),
        // structs
        ("make-st", Some(Scope::Extern), 2, Struct::mu_make_struct),
        ("st-type", Some(Scope::Extern), 1, Struct::mu_struct_type),
        ("st-vec", Some(Scope::Extern), 1, Struct::mu_struct_vector),
        // streams
        ("close", Some(Scope::Extern), 1, Stream::mu_close),
        ("eof", Some(Scope::Extern), 1, Stream::mu_eof),
        ("flush", Some(Scope::Extern), 1, Stream::mu_flush),
        ("get-str", Some(Scope::Extern), 1, Stream::mu_get_string),
        ("open", Some(Scope::Extern), 3, Stream::mu_open),
        ("openp", Some(Scope::Extern), 1, Stream::mu_openp),
        ("rd-byte", Some(Scope::Extern), 3, Stream::mu_read_byte),
        ("rd-char", Some(Scope::Extern), 3, Stream::mu_read_char),
        ("un-char", Some(Scope::Extern), 2, Stream::mu_unread_char),
        ("wr-byte", Some(Scope::Extern), 2, Stream::mu_write_byte),
        ("wr-char", Some(Scope::Extern), 2, Stream::mu_write_char),
        // interns
        ("bq-append", None, 2, Mu::_append),
        ("if", None, 3, Mu::_if),
    ];
}

pub trait Core {
    fn map_internal(_: &Mu, name: String) -> Option<Tag>;
    fn install_lib_functions(_: &Mu) -> (Vec<LibFunction>, Vec<InternalFunction>);
}

impl Core for Mu {
    fn map_internal(mu: &Mu, name: String) -> Option<Tag> {
        mu.internals
            .iter()
            .find(|(libname, _)| *libname == name)
            .map(|(_, libfn)| *libfn)
    }

    fn install_lib_functions<'a>(mu: &Mu) -> (Vec<LibFunction>, Vec<InternalFunction>) {
        Namespace::intern(
            mu,
            mu.mu_ns,
            Scope::Extern,
            "version".to_string(),
            mu.version,
        );
        Namespace::intern(mu, mu.mu_ns, Scope::Extern, "std-in".to_string(), mu.stdin);
        Namespace::intern(
            mu,
            mu.mu_ns,
            Scope::Extern,
            "std-out".to_string(),
            mu.stdout,
        );
        Namespace::intern(
            mu,
            mu.mu_ns,
            Scope::Extern,
            "err-out".to_string(),
            mu.errout,
        );

        let mut funcv = Vec::new();
        let mut internals = Vec::new();

        for (id, fnmap) in SYMBOLMAP.iter().enumerate() {
            let (name, scope, nreqs, libfn) = fnmap;

            let func = Function::new(
                Fixnum::as_tag(*nreqs as i64),
                Fixnum::as_tag(match id.try_into().unwrap() {
                    Some(n) => n as i64,
                    None => panic!(),
                }),
                Symbol::new(mu, Tag::nil(), Scope::Extern, name, Tag::nil()).evict(mu),
            )
            .evict(mu);

            funcv.push(*libfn);

            match scope {
                Some(scope) => {
                    Namespace::intern(mu, mu.mu_ns, *scope, name.to_string(), func);
                }
                None => {
                    internals.push((name.to_string(), func));
                }
            };
        }

        (funcv, internals)
    }
}

pub trait MuFunction {
    fn mu_apply(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_compile(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_eval(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_exit(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_fix(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn _if(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_real_time(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_run_time(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_view(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn mu_write(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn mu_real_time(_: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match System::real_time() {
            Ok(us) => Fixnum::as_tag(us as i64),
            Err(_) => return Err(Exception::new(Condition::Error, "mu:real-us", Tag::nil())),
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

        fp.value = match Tag::type_of(mu, func) {
            Type::Function => match Tag::type_of(mu, args) {
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
                _ => return Err(Exception::new(Condition::Type, "mu:apply", args)),
            },
            _ => return Err(Exception::new(Condition::Type, "mu:apply", func)),
        };

        Ok(())
    }

    fn mu_write(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let form = fp.argv[0];
        let escape = fp.argv[1];
        let stream = fp.argv[2];

        fp.value = form;

        match Tag::type_of(mu, stream) {
            Type::Stream => match mu.write(form, !escape.null_(), stream) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            _ => Err(Exception::new(Condition::Type, "mu:write", stream)),
        }
    }

    fn _if(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let test = fp.argv[0];
        let true_fn = fp.argv[1];
        let false_fn = fp.argv[2];

        fp.value = match Tag::type_of(mu, true_fn) {
            Type::Function => match Tag::type_of(mu, false_fn) {
                Type::Function => {
                    match mu.apply(if test.null_() { false_fn } else { true_fn }, Tag::nil()) {
                        Ok(tag) => tag,
                        Err(e) => return Err(e),
                    }
                }
                _ => return Err(Exception::new(Condition::Type, "mu::if", false_fn)),
            },
            _ => return Err(Exception::new(Condition::Type, "mu::if", true_fn)),
        };

        Ok(())
    }

    fn mu_exit(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let rc = fp.argv[0];

        match Tag::type_of(mu, rc) {
            Type::Fixnum => std::process::exit(Fixnum::as_i64(mu, rc) as i32),
            _ => Err(Exception::new(Condition::Type, "mu:exit", rc)),
        }
    }

    fn mu_view(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let tag = fp.argv[0];

        fp.value = match Tag::type_of(mu, tag) {
            Type::Char => Char::view(mu, tag),
            Type::Cons => Cons::view(mu, tag),
            Type::Fixnum => Fixnum::view(mu, tag),
            Type::Float => Float::view(mu, tag),
            Type::Function => Function::view(mu, tag),
            Type::Namespace => Namespace::view(mu, tag),
            Type::Null | Type::Symbol | Type::Keyword => Symbol::view(mu, tag),
            Type::Stream => Stream::view(mu, tag),
            Type::Struct => Struct::view(mu, tag),
            Type::Vector => Vector::view(mu, tag),
            _ => return Err(Exception::new(Condition::Type, "mu:view", tag)),
        };

        Ok(())
    }

    fn mu_fix(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let func = fp.argv[0];

        fp.value = fp.argv[1];

        match Tag::type_of(mu, func) {
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
            _ => Err(Exception::new(Condition::Type, "mu:fix", func)),
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
