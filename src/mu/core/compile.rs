//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! compile:
//!     function calls
//!     special forms
use crate::{
    core::{
        exception::{self, Condition, Exception},
        frame::Frame,
        mu::{Core as _, Mu},
        namespace::Core,
        types::{Tag, Type},
    },
    types::{
        cons::{Cons, ConsIter, Core as _},
        fixnum::Fixnum,
        function::Function,
        symbol::{Core as _, Symbol},
    },
};

use crate::core::async_context::{AsyncContext, Core as _};
use futures::executor::block_on;

// special forms
type SpecFn = fn(&Mu, Tag) -> exception::Result<Tag>;
type SpecMap = (Tag, SpecFn);

lazy_static! {
    static ref SPECMAP: Vec<SpecMap> = vec![
        (Symbol::keyword("async"), Mu::compile_async),
        (Symbol::keyword("if"), Mu::compile_if),
        (Symbol::keyword("lambda"), Mu::compile_lambda),
        (Symbol::keyword("quote"), Mu::compile_quoted_list),
    ];
}

pub trait Compiler {
    fn compile(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_async(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_if(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_lambda(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_lexical(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_list(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_quoted_list(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_special_form(_: &Mu, _: Tag, args: Tag) -> exception::Result<Tag>;
}

impl Compiler for Mu {
    fn compile_if(mu: &Mu, args: Tag) -> exception::Result<Tag> {
        if Cons::length(mu, args) != Some(3) {
            return Err(Exception::new(Condition::Syntax, ":if", args));
        }

        let if_fn = <Mu as Core>::intern_symbol(mu, mu.mu_ns, "%if".to_string(), Tag::nil());
        let lambda = Symbol::keyword("lambda");

        let if_vec = vec![
            if_fn,
            match Cons::nth(mu, 0, args) {
                Some(t) => t,
                None => panic!(),
            },
            Cons::vlist(
                mu,
                &[
                    lambda,
                    Tag::nil(),
                    match Cons::nth(mu, 1, args) {
                        Some(t) => t,
                        None => panic!(),
                    },
                ],
            ),
            Cons::vlist(
                mu,
                &[
                    lambda,
                    Tag::nil(),
                    match Cons::nth(mu, 2, args) {
                        Some(t) => t,
                        None => panic!(),
                    },
                ],
            ),
        ];

        Self::compile(mu, Cons::vlist(mu, &if_vec))
    }

    fn compile_quoted_list(mu: &Mu, list: Tag) -> exception::Result<Tag> {
        if Cons::length(mu, list) != Some(1) {
            return Err(Exception::new(Condition::Syntax, ":quote", list));
        }

        Ok(Cons::new(Symbol::keyword("quote"), list).evict(mu))
    }

    fn compile_special_form(mu: &Mu, name: Tag, args: Tag) -> exception::Result<Tag> {
        match SPECMAP.iter().copied().find(|spec| name.eq_(spec.0)) {
            Some(spec) => spec.1(mu, args),
            None => Err(Exception::new(Condition::Syntax, "specf", args)),
        }
    }

    // utilities
    fn compile_list(mu: &Mu, body: Tag) -> exception::Result<Tag> {
        let mut body_vec = Vec::new();

        for cons in ConsIter::new(mu, body) {
            match Self::compile(mu, Cons::car(mu, cons)) {
                Ok(expr) => body_vec.push(expr),
                Err(e) => return Err(e),
            }
        }

        Ok(Cons::vlist(mu, &body_vec))
    }

    fn compile_lambda(mu: &Mu, args: Tag) -> exception::Result<Tag> {
        fn compile_frame_symbols(mu: &Mu, lambda: Tag) -> exception::Result<Vec<Tag>> {
            let mut symvec = Vec::new();

            for cons in ConsIter::new(mu, lambda) {
                let symbol = Cons::car(mu, cons);
                if Tag::type_of(symbol) == Type::Symbol {
                    match symvec.iter().rev().position(|lex| symbol.eq_(*lex)) {
                        Some(_) => {
                            return Err(Exception::new(Condition::Syntax, "lexical", symbol))
                        }
                        _ => symvec.push(symbol),
                    }
                } else {
                    return Err(Exception::new(Condition::Type, "lexical", symbol));
                }
            }

            Ok(symvec)
        }

        let (lambda, body) = match Tag::type_of(args) {
            Type::Cons => {
                let lambda = Cons::car(mu, args);

                match Tag::type_of(lambda) {
                    Type::Null | Type::Cons => (lambda, Cons::cdr(mu, args)),
                    _ => return Err(Exception::new(Condition::Type, "lambda", args)),
                }
            }
            _ => return Err(Exception::new(Condition::Syntax, "lambda", args)),
        };

        let func = Function::new(
            Fixnum::as_tag(Cons::length(mu, lambda).unwrap() as i64),
            Tag::nil(),
        )
        .evict(mu);

        match compile_frame_symbols(mu, lambda) {
            Ok(lexicals) => {
                let mut lexenv_ref = block_on(mu.compile.write());

                lexenv_ref.push((func, lexicals));
            }
            Err(e) => return Err(e),
        };

        let form = match Self::compile_list(mu, body) {
            Ok(form) => {
                let mut function = Function::to_image(mu, func);
                function.form = form;
                Function::update(mu, &function, func);

                Ok(func)
            }
            Err(e) => Err(e),
        };

        let mut lexenv_ref = block_on(mu.compile.write());

        lexenv_ref.pop();

        form
    }

    fn compile_async(mu: &Mu, args: Tag) -> exception::Result<Tag> {
        let (func, arg_list) = match Tag::type_of(args) {
            Type::Cons => {
                let fn_arg = match Self::compile(mu, Cons::car(mu, args)) {
                    Ok(fn_) => match Tag::type_of(fn_) {
                        Type::Function => fn_,
                        Type::Symbol => {
                            let sym_val = Symbol::value(mu, fn_);
                            match Tag::type_of(sym_val) {
                                Type::Function => sym_val,
                                _ => return Err(Exception::new(Condition::Type, "async", sym_val)),
                            }
                        }
                        _ => return Err(Exception::new(Condition::Type, "async", fn_)),
                    },
                    Err(e) => return Err(e),
                };

                let async_args = match Self::compile_list(mu, Cons::cdr(mu, args)) {
                    Ok(list) => list,
                    Err(e) => return Err(e),
                };

                let arity = Fixnum::as_i64(Function::arity(mu, fn_arg));
                if arity != Cons::length(mu, async_args).unwrap() as i64 {
                    return Err(Exception::new(Condition::Arity, "async", args));
                }

                (fn_arg, async_args)
            }
            _ => return Err(Exception::new(Condition::Syntax, "async", args)),
        };

        match AsyncContext::async_context(mu, func, arg_list) {
            Ok(asyncid) => Ok(asyncid),
            Err(e) => Err(e),
        }
    }

    fn compile_lexical(mu: &Mu, symbol: Tag) -> exception::Result<Tag> {
        let lexenv_ref = block_on(mu.compile.read());

        for frame in lexenv_ref.iter().rev() {
            let (tag, symbols) = frame;

            if let Some(nth) = symbols.iter().position(|lex| symbol.eq_(*lex)) {
                let lex_ref = vec![
                    <Mu as Core>::intern_symbol(mu, mu.mu_ns, "fr-ref".to_string(), Tag::nil()),
                    Fixnum::as_tag(tag.as_u64() as i64),
                    Fixnum::as_tag(nth as i64),
                ];

                match Self::compile(mu, Cons::vlist(mu, &lex_ref)) {
                    Ok(lexref) => return Ok(lexref),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(symbol)
    }

    fn compile(mu: &Mu, expr: Tag) -> exception::Result<Tag> {
        match Tag::type_of(expr) {
            Type::Symbol => Self::compile_lexical(mu, expr),
            Type::Cons => {
                let func = Cons::car(mu, expr);
                let args = Cons::cdr(mu, expr);
                match Tag::type_of(func) {
                    Type::Keyword => match Self::compile_special_form(mu, func, args) {
                        Ok(form) => Ok(form),
                        Err(e) => Err(e),
                    },
                    Type::Symbol | Type::Function => match Self::compile_list(mu, args) {
                        Ok(arglist) => Ok(Cons::new(func, arglist).evict(mu)),

                        Err(e) => Err(e),
                    },
                    Type::Cons => match Self::compile_list(mu, args) {
                        Ok(arglist) => match Self::compile(mu, func) {
                            Ok(fnc) => match Tag::type_of(fnc) {
                                Type::Function => Ok(Cons::new(fnc, arglist).evict(mu)),
                                _ => Err(Exception::new(Condition::Type, "compile", func)),
                            },
                            Err(e) => Err(e),
                        },
                        Err(e) => Err(e),
                    },
                    _ => Err(Exception::new(Condition::Type, "compile", func)),
                }
            }
            _ => Ok(expr),
        }
    }
}

pub trait MuFunction {
    fn mu_compile(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn if_(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
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

    fn mu_compile(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match <Mu as Compiler>::compile(mu, fp.argv[0]) {
            Ok(tag) => tag,
            Err(e) => return Err(e),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        compile::Compiler,
        config::{self, Config},
        mu::{Core, Mu},
        types::{Tag, Type},
    };

    #[test]
    fn compile_test() {
        let config = match <Config as config::Core>::config("".to_string()) {
            Some(config) => config,
            None => return assert!(false),
        };

        let mu: &Mu = &Core::new(&config);

        match <Mu as Compiler>::compile(mu, Tag::nil()) {
            Ok(form) => match Tag::type_of(form) {
                Type::Null => assert!(true),
                _ => assert!(false),
            },
            _ => assert!(false),
        }
        match <Mu as Compiler>::compile_list(mu, Tag::nil()) {
            Ok(form) => match Tag::type_of(form) {
                Type::Null => assert!(true),
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }
}
