;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; core compiler
;;;

;;;
;;; special forms
;;;

;;;
;;; compile a lambda form to a function core-type
;;;
;;; returns the compiled function
;;;
(mu:intern :core "%compile-lambda"
   (:lambda (form env)
      (:if (mu:eq (mu:length form) 1)
           (core:raise form 'core:%compile-defmacro "malformed lambda expression")
           ((:lambda (lambda body)
               ((:lambda (lambda-desc)
                   ((:lambda (compiled-function)
                       (:if (core:%orf
                             (core:%lambda-prop :lexenv lambda-desc)
                             (core:%lambda-prop :rest lambda-desc))
                            (core:%make-funct lambda-desc compiled-function ())
                            compiled-function))
                    (mu:compile
                     `(:lambda
                       ,(core:%lambda-prop :bound lambda-desc)
                       ,@(core:%compile-lambda-body
                          lambda-desc
                          body
                          env)))))
                (core:%compile-core-lambda lambda env)))
            (mu:nth 1 form)
            (mu:nthcdr 2 form)))))

;;;
;;; compile a defmacro form to a bound macro symbol
;;;
;;; returns the macro symbol
;;;
(mu:intern :core "%compile-defmacro"
   (:lambda (form env)
      ((:lambda (macro-symbol lambda-form)
          (:if (core:symbolp macro-symbol)
               (core:%macro-env macro-symbol (core:%compile-lambda lambda-form env))
               (core:raise macro-symbolsymbol 'core:%compile-defmacro "not a symbol"))
          (core:%list-2 :quote macro-symbol))
       (mu:nth 1 form)
       (mu:nthcdr 1 form))))

;;;
;;; this if form can take two or three arguments
;;;
;;; returns an application of the if form
;;;
(mu:intern :core "%thunk"
  (:lambda (form env)
    (core:%compile
     `(:lambda () ,form)
     env)))

(mu:intern :core "%compile-if"
   (:lambda (form env)
      ((:lambda (length)
          (:if (core:%orf (mu:eq length 3) (mu:eq length 4))
               ((:lambda (t-arm f-arm)
                  (mu:compile `(:if ,(mu:nth 1 form) ,t-arm ,f-arm)))
               (:if (core:constantp (mu:nth 2 form))
                    (mu:nth 2 form)
                    `(core:apply ,(core:%thunk (mu:nth 2 form) env) ()))
               (:if (core:constantp (mu:nth 3 form))
                    (mu:nth 3 form)
                    `(core:apply ,(core:%thunk (mu:nth 3 form) env) ())))
              (core:raise form 'core:%compile-if "takes two or three arguments")))
       (mu:length form))))

;;;
;;; core compiler
;;;
;;; rewrites a function call/special call/macro call against an environment
;;; into something the runtime can execute. constants compile to themselves.
;;;
;;; returns a mu form or funct ctype (in the case of a define-core-lambda)
;;;
(mu:intern :core "%compile"
   (:lambda (form env)
      (:if (core:consp form)
           ((:lambda (function-form arg-list)
               (:if (core:keywordp function-form)
                    (mu:compile form)                    ; mu special form (:key ...)
                    (:if (mu:eq :symbol (mu:type-of function-form)) ; special form or macro
                         ((:lambda (special)
                             (:if special
                                  (mu:apply (mu:sy-val special) (core:%list-2 form env))
                                  ((:lambda (macro-function)
                                      (:if macro-function
                                           (core:%compile-macro-call function-form arg-list env)
                                           (core:%compile-funcall function-form arg-list env)))
                                   (core:macro-function function-form env))))
                          (mu:cdr (core:%assoc
                                   function-form
                                   '((define-macro . core:%compile-defmacro)
                                     (if           . core:%compile-if)
                                     (lambda       . core:%compile-lambda)))))
                    (core:%compile-funcall function-form arg-list env))))
            (mu:car form)
            (mu:cdr form))
           form)))

;;;
;;; the external compile interface, compiles in
;;; the null environment
;;;
(mu:intern :core "compile"
   (:lambda (form)
     (core:%compile form ())))
