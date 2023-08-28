;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; macro definition, compilation, and expansion
;;;

;;;
;;; Macro definitions are compiled like lambdas (though eventually with a slighlty
;;; different lambda list including &whole).
;;;
;;; Macro calls are expanded at compile time.
;;;
;;; Common Lisp adds an environment argument here for lexial macro expanders.
;;;
;;; an environment is a closure that evaluates its
;;; arguments, as soon as we get closures working
;;;

;;;
;;; macro environment
;;;
(mu:intern core "%core-macro-env"
  (:lambda (symbol macro-fn)
    ((:lambda (macro-ns-name)
       ((:lambda (macro-ns symbol-ns symbol-name)
          (:if macro-ns
               (mu:intern (mu:sy-val macro-ns) symbol-name macro-fn)
               ((:lambda (ns-sym)
                  (mu:intern (mu:sy-val ns-sym) symbol-name macro-fn))
                (mu:intern
                 symbol-ns
                 macro-ns-name
                 (mu:make-ns macro-ns-name ())))))
        (mu:ns-find (core:symbol-ns symbol) macro-ns-name)
        (core:symbol-ns symbol)
        (mu:sy-name symbol)))
     (core:string-append `(,(mu:ns-name (core:symbol-ns symbol)) ":macrodefs")))))

(mu:intern core "%core-find-macro"
   (:lambda (symbol env)
      (:if (mu:eq :symbol (mu:type-of symbol))
           ((:lambda (macro-ns symbol-ns symbol-name)
                     (:if macro-ns
                          (mu:ns-find (mu:sy-val macro-ns) symbol-name)
                          ()))
            (mu:ns-find
             (core:symbol-ns symbol)
             (core:string-append `(,(mu:ns-name (core:symbol-ns symbol)) ":macrodefs")))
            (core:symbol-ns symbol)
            (mu:sy-name symbol))
           (core:raise symbol 'core:%core-find-macro "is not a symbol"))))

;;;
;;; macroexpand-1 expands a macro call once.
;;;
(mu:intern core "macroexpand-1"
   (:lambda (form env)
     (:if (core:consp form)
          ((:lambda (fn-symbol args)
             (:if (mu:eq :symbol (mu:type-of fn-symbol))
                  ((:lambda (macro-fn)
                     (:if macro-fn
                          (core:apply macro-fn args)
                          form))
                   (core:macro-function fn-symbol env))
                  form))
           (mu:car form)
           (mu:cdr form))
          form)))

;;;
;;; macroexpand: expand a form until it is no longer a macro call
;;;
(mu:intern core "macroexpand"
   (:lambda (form env)
     ((:lambda (expanded)
          (:if (mu:eq form expanded)
             expanded
             (core:macroexpand expanded env)))
       (core:macroexpand-1 form env))))

;;;
;;; macro-function: map a symbol to a macro function
;;;
;;; acts as a predicate
;;;
(mu:intern core "macro-function"
  (:lambda (symbol env)
     (:if (mu:eq :symbol (mu:type-of symbol))
          (:if (core:keywordp symbol)
               ()
               ((:lambda (macro)
                  (:if macro
                       (mu:sy-val macro)
                       ()))
                (core:%core-find-macro symbol env)))
          ())))

;;;
;;; %compile-macro lambda env => function
;;;
;;; %compile-macro compiles a macro expander function.
;;;
(mu:intern core "%core-macro"
   (:lambda (lambda env)
     (:if (core:findl (:lambda (el) (core:null (mu:eq :symbol (mu:type-of el)))) lambda)
         (core:raise-if lambda 'core:%core-macro "lambda syntax")
         ((:lambda (desc)
             (:if (core:fixnump desc)
                  (core:%make-lambda lambda () env)
                  desc))
          (core:%foldl
            (:lambda (el acc)
               (:if (core:structp acc)
                  acc
                  (:if (mu:eq '&rest el)
                     (:if (mu:eq (mu:length lambda) (mu:fx-add 2 acc))
                          (core:%make-lambda
                           `(,@(core:dropr lambda 2) ,@(core:dropl lambda (core:1- (mu:length lambda))))
                           (mu:nth (core:1- (mu:length lambda)) lambda)
                           env)
                          (core:raise lambda 'core:%core-macro "rest botch"))
                     (core:1+ acc))))
            0
            lambda)))))

(mu:intern core "%compile-macro"
  (:lambda (form env)
    ((:lambda (lambda-desc body)
       (mu:compile
        `(:lambda ,(core:%lambda-prop :bound lambda-desc)
                  ,@(core:%compile-lambda-body lambda-desc body env))))
     (core:%core-macro (mu:car form) env)
     (mu:nthcdr 1 form))))
