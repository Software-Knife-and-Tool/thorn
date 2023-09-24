;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; core compiler
;;;

;;;
;;; special forms
;;;

;;;
;;; lambda compiles to a function core-type
;;;
(mu:intern core "%compile-lambda"
   (:lambda (form env)
      (:if (mu:eq (mu:length form) 1)
           (core:raise form 'core:%compile-defmacro "malformed lambda expression")
           ((:lambda (lambda body)
               ((:lambda (lambda-desc)
                   ((:lambda (fn)
                       (:if (core:%orf
                             (core:%lambda-prop :lexenv lambda-desc)
                             (core:%lambda-prop :rest lambda-desc))
                            (core:%make-function lambda-desc fn ())
                            fn))
                    (mu:compile
                     `(:lambda
                       ,(core:%lambda-prop :bound lambda-desc)
                       ,@(core:%compile-lambda-body
                          lambda-desc
                          body
                          env)))))
                (core:%core-lambda lambda env)))
            (mu:nth 1 form)
            (mu:nthcdr 2 form)))))

;;;
;;; defmacro compiles to a bound macro symbol
;;;
(mu:intern core "%compile-defmacro"
   (:lambda (form env)
      ((:lambda (symbol lambda-form)
          (:if (core:symbolp symbol)
               (core:%core-macro-env symbol (core:%compile-lambda lambda-form env))
               (core:raise symbol 'core:%compile-defmacro "not a symbol"))
          `(:quote ,symbol))
       (mu:nth 1 form)
       (mu:nthcdr 1 form))))

;;;
;;; defun compiles to a bound function symbol
;;;
(mu:intern core "%compile-defun"
   (:lambda (form env)
      ((:lambda (symbol lambda-form)
          (:if (mu:eq (mu:type-of symbol) :symbol)
               (:if (core:listp lambda-form)
                    (core:intern symbol (core:%compile-lambda lambda-form env))
                    (core:raise lambda-form 'core:%compile-defun "not a lambda list"))
               (core:raise symbol 'core:%compile-defun "not a symbol"))
          `(:quote ,symbol))
       (mu:nth 1 form)
       (mu:nthcdr 1 form))))

;;;
;;; this if form can take two or three arguments
;;;
(mu:intern core "%thunk"
  (:lambda (form env)
    (core:%compile
     `(:lambda () ,form)
     env)))

(mu:intern core "%compile-if"
   (:lambda (form env)
      ((:lambda (len)
          (:if (core:%orf (mu:eq len 3) (mu:eq len 4))
               ((:lambda (t-arm f-arm)
                  (mu:compile `(:if ,(mu:nth 1 form) ,t-arm ,f-arm)))
                  ;;; (mu:compile `(:if ,(mu:cons (core:%thunk (mu:nth 1 form) env) ()) ,t-arm ,f-arm)))
               (:if (core:constantp (mu:nth 2 form))
                    (mu:nth 2 form)
                    `(core:apply ,(core:%thunk (mu:nth 2 form) env) ()))
               (:if (core:constantp (mu:nth 3 form))
                    (mu:nth 3 form)
                    `(core:apply ,(core:%thunk (mu:nth 3 form) env) ())))
              (core:raise form 'core:%compile-if "takes two or three arguments")))
       (mu:length form))))

;;;
;;; compiler
;;;
;;; rewrites a funcall/special/macro call against an environment
;;; into something the runtime can execute. constants compile
;;; to themselves.
;;;
;;; arguments:
;;;
;;;   form : function call, macro call, special form call, or constant
;;;   env  : lexical environment list
;;;
;;; returns: mu form or funct ctype (in the case of a define-core-lambda)
;;;
(mu:intern core "%compile"
   (:lambda (form env)
      (:if (core:consp form)
           ((:lambda (fn args)
               (:if (core:keywordp fn)
                    (mu:compile form)                    ; mu special form (:key ...)
                    (:if (mu:eq :symbol (mu:type-of fn)) ; special form or macro
                         ((:lambda (special)
                             (:if special
                                  (mu:apply (mu:sy-val special) `(,form ,env))
                                  ((:lambda (macro-fn)
                                      (:if macro-fn
                                           (core:%compile-macro-call fn args env)
                                           (core:%compile-funcall fn args env)))
                                   (core:macro-function fn env))))
                          (mu:cdr (core:%assoc
                                   fn
                                   '((define-core-function . core:%compile-defun)
                                     (define-core-macro    . core:%compile-defmacro)
                                     (define-core-if       . core:%compile-if)
                                     (define-core-lambda   . core:%compile-lambda)))))
                    (core:%compile-funcall fn args env))))
            (mu:car form)
            (mu:cdr form))
           form)))

(mu:intern core "compile"
   (:lambda (form)
     (core:%compile form ())))
