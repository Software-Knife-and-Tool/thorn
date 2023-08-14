//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! compile:
//!     function calls
//!     special forms
use crate::{
    core::{
        exception::{self, Condition, Exception},
        functions::Core as _,
        mu::Mu,
        types::{Tag, Type},
    },
    types::{
        cons::{Cons, ConsIter, Core as _},
        fixnum::Fixnum,
        function::Function,
        namespace::{Core as _, Namespace, Scope},
        symbol::{Core as _, Symbol},
    },
};

// special forms
type SpecFn = fn(&Mu, Tag) -> exception::Result<Tag>;
type SpecMap = (Tag, SpecFn);

lazy_static! {
    static ref SPECMAP: Vec<SpecMap> = vec![
        (Symbol::keyword("if"), Mu::compile_if),
        (Symbol::keyword("lambda"), Mu::compile_lambda),
        (Symbol::keyword("quote"), Mu::compile_quoted_list),
    ];
}

pub trait Compiler {
    fn compile(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_frame_symbols(_: &Mu, _: Tag) -> exception::Result<Vec<Tag>>;
    fn compile_if(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_lambda(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_lexical(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_quoted_list(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_list(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn compile_special_form(_: &Mu, _: Tag, args: Tag) -> exception::Result<Tag>;
}

impl Compiler for Mu {
    fn compile_if(mu: &Mu, args: Tag) -> exception::Result<Tag> {
        if Cons::length(mu, args) != Some(3) {
            return Err(Exception::new(Condition::Syntax, ":if", args));
        }

        let if_fn = match Mu::map_internal(mu, "if".to_string()) {
            Some(fn_) => fn_,
            None => panic!(),
        };

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

    // lexical symbols
    fn compile_frame_symbols(mu: &Mu, lambda: Tag) -> exception::Result<Vec<Tag>> {
        let mut symv = Vec::new();

        for cons in ConsIter::new(mu, lambda) {
            let symbol = Cons::car(mu, cons);
            if Tag::type_of(mu, symbol) == Type::Symbol {
                match symv.iter().rev().position(|lex| symbol.eq_(*lex)) {
                    Some(_) => return Err(Exception::new(Condition::Syntax, "lexical", symbol)),
                    _ => symv.push(symbol),
                }
            } else {
                return Err(Exception::new(Condition::Type, "lexical", symbol));
            }
        }

        Ok(symv)
    }

    fn compile_lexical(mu: &Mu, symbol: Tag) -> exception::Result<Tag> {
        let lexenv_ref = mu.compile.read().unwrap();

        for frame in lexenv_ref.iter().rev() {
            let (tag, symbols) = frame;

            if let Some(nth) = symbols.iter().position(|lex| symbol.eq_(*lex)) {
                let lex_ref = vec![
                    Namespace::mu_int_symbol(mu, "fr-ref".to_string()),
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

    fn compile_lambda(mu: &Mu, args: Tag) -> exception::Result<Tag> {
        let (lambda, body) = match Tag::type_of(mu, args) {
            Type::Cons => {
                let lambda = Cons::car(mu, args);

                match Tag::type_of(mu, lambda) {
                    Type::Null | Type::Cons => (lambda, Cons::cdr(mu, args)),
                    _ => return Err(Exception::new(Condition::Type, "lambda", args)),
                }
            }
            _ => return Err(Exception::new(Condition::Syntax, "lambda", args)),
        };

        let id = Symbol::new(mu, Tag::nil(), Scope::Extern, "lambda", Tag::nil()).evict(mu);

        match Self::compile_frame_symbols(mu, lambda) {
            Ok(lexicals) => {
                let mut lexenv_ref = mu.compile.write().unwrap();
                lexenv_ref.push((id, lexicals));
            }
            Err(e) => return Err(e),
        };

        let form = match Self::compile_list(mu, body) {
            Ok(form) => match Cons::length(mu, lambda) {
                Some(len) => Ok(Function::new(Fixnum::as_tag(len as i64), form, id).evict(mu)),
                None => panic!(":lambda"),
            },
            Err(e) => Err(e),
        };

        let mut lexenv_ref = mu.compile.write().unwrap();
        lexenv_ref.pop();

        form
    }

    fn compile(mu: &Mu, expr: Tag) -> exception::Result<Tag> {
        match Tag::type_of(mu, expr) {
            Type::Symbol => Self::compile_lexical(mu, expr),
            Type::Cons => {
                let func = Cons::car(mu, expr);
                let args = Cons::cdr(mu, expr);
                match Tag::type_of(mu, func) {
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
                            Ok(fnc) => match Tag::type_of(mu, fnc) {
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

#[cfg(test)]
mod tests {
    use crate::core::{
        compile::Compiler,
        mu::{Core, Mu},
        types::{Tag, Type},
    };

    #[test]
    fn compile_test() {
        let mu: &Mu = &Core::new("".to_string());

        match <Mu as Compiler>::compile(mu, Tag::nil()) {
            Ok(form) => match Tag::type_of(mu, form) {
                Type::Null => assert!(true),
                _ => assert!(false),
            },
            _ => assert!(false),
        }
        match <Mu as Compiler>::compile_list(mu, Tag::nil()) {
            Ok(form) => match Tag::type_of(mu, form) {
                Type::Null => assert!(true),
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }
}
