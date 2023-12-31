;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; prelude compiler
;;;

;;;
;;; special forms
;;;

;;;
;;; compile a lambda form to a function prelude type
;;;
;;; returns the compiled function
;;;
(mu:intern :prelude "%compile-lambda"
   (:lambda (form env)
      (:if (mu:eq (mu:length form) 1)
           (prelude:raise form 'prelude:%compile-lambda "malformed lambda expression")
           ((:lambda (lambda body)
               ((:lambda (lambda-desc)
                   ((:lambda (compiled-function)
                       (:if (prelude:%orf
                             (prelude:%lambda-prop :lexenv lambda-desc)
                             (prelude:%lambda-prop :rest lambda-desc))
                            (prelude:%make-funct lambda-desc compiled-function ())
                            compiled-function))
                    (mu:compile
                     `(:lambda
                       ,(prelude:%lambda-prop :bound lambda-desc)
                       ,@(prelude:%compile-lambda-body
                          lambda-desc
                          body
                          env)))))
                (prelude:%compile-lambda-list lambda env)))
            (mu:nth 1 form)
            (mu:nthcdr 2 form)))))

;;;
;;; compile a defmacro form to a bound macro symbol
;;;
;;; returns the macro symbol
;;;
(mu:intern :prelude "%compile-defmacro"
   (:lambda (form env)
      ((:lambda (macro-symbol lambda-form)
          (:if (prelude:symbolp macro-symbol)
               (prelude:%defmacro macro-symbol (prelude:%compile-lambda lambda-form env))
               (prelude:raise macro-symbol 'prelude:%compile-defmacro "not a symbol"))
          (prelude:%list-2 :quote macro-symbol))
       (mu:nth 1 form)
       (mu:nthcdr 1 form))))

;;;
;;; this if form can take two or three arguments
;;;
;;; returns an application of the if form
;;;
(mu:intern :prelude "%make-thunk"
  (:lambda (form env)
    (prelude:%compile
     `(:lambda () ,form)
     env)))

(mu:intern :prelude "%compile-if"
   (:lambda (form env)
      ((:lambda (length)
          (:if (prelude:%orf (mu:eq length 3) (mu:eq length 4))
               ((:lambda (t-arm f-arm)
                  (mu:compile `(:if ,(mu:nth 1 form) ,t-arm ,f-arm)))
                (:if (prelude:constantp (mu:nth 2 form))
                     (mu:nth 2 form)
                     `(prelude:apply ,(prelude:%make-thunk (mu:nth 2 form) env) ()))
                (:if (prelude:constantp (mu:nth 3 form))
                     (mu:nth 3 form)
                     `(prelude:apply ,(prelude:%make-thunk (mu:nth 3 form) env) ())))
               (prelude:raise form 'prelude:%compile-if "takes two or three arguments")))
       (mu:length form))))

;;;
;;; prelude compiler
;;;
;;; rewrites a function call/special call/macro call against an environment
;;; into something the runtime can execute. constants compile to themselves.
;;;
;;; returns a mu form or funct type (in the case of a define-prelude-lambda)
;;;
(mu:intern :prelude "%compile"
   (:lambda (form env)
      (:if (prelude:consp form)
           ((:lambda (function-form arg-list)
               (:if (prelude:keywordp function-form)
                    (mu:compile form)                    ; mu special form (:key ...)
                    (:if (mu:eq :symbol (mu:type-of function-form)) ; special form or macro
                         ((:lambda (special)
                             (:if special
                                  (mu:apply (mu:sy-val special) (prelude:%list-2 form env))
                                  ((:lambda (macro-function)
                                      (:if macro-function
                                           (prelude:%compile-macro-call function-form arg-list env)
                                           (prelude:%compile-funcall function-form arg-list env)))
                                   (prelude:macro-function function-form env))))
                          (mu:cdr (prelude:%assoc
                                   function-form
                                   '((define-macro . prelude:%compile-defmacro)
                                     (if           . prelude:%compile-if)
                                     (lambda       . prelude:%compile-lambda)))))
                    (prelude:%compile-funcall function-form arg-list env))))
            (mu:car form)
            (mu:cdr form))
           form)))

;;;
;;; the external compile interface, compiles in
;;; the null environment
;;;
(mu:intern :prelude "compile"
   (:lambda (form)
     (prelude:%compile form ())))
