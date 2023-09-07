;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; core compiler
;;;

;;;
;;; compiling special/macro forms
;;;
;;;    core:defmacro-compiler form env => list
;;;    core:defun-compiler form env => list
;;;    core:if-compiler form env => list
;;;    core:lambda-compiler form env => list
;;;    core:quote-compiler form env => list
;;;
;;; special/macro form compiler rewrites a form against
;;; an environment into something the runtime can execute.
;;;
;;; arguments:
;;;
;;;   form : special/macro form
;;;   env  : lexical environment list
;;;
;;; returns: mu form or closure
;;;

;;;
;;; special forms
;;;
(mu:intern core "%core-special-table"
   (:lambda (symbol)
      (mu:cdr (core:%assoc
        symbol
        '((defun    . core:defun-compiler)
          (defmacro . core:defmacro-compiler)
          (if       . core:if-compiler)
          (lambda   . core:lambda-compiler)
          (quote    . core:quote-compiler))))))

;;;
;;; lambda compiles to a closure core-type
;;;
(mu:intern core "lambda-compiler"
   (:lambda (form env)
      (:if (mu:eq (mu:length form) 1)
           (core:raise form 'core:lambda-compiler "malformed lambda expression")           
           ((:lambda (lambda body)
               ((:lambda (lambda-desc)
                   ((:lambda (fn)
                       (:if (core:%orf
                             (core:%lambda-prop :lexenv lambda-desc)
                             (core:%lambda-prop :rest lambda-desc))
                            (core:%make-closure lambda-desc fn ())
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
(mu:intern core "defmacro-compiler"
   (:lambda (form env)
      ((:lambda (symbol lambda-form)
          (:if (core:symbolp symbol)
               (core:intern symbol (core:%core-macro-env symbol (core:%compile-macro lambda-form env)))
               (core:raise symbol 'core:defmacro-compiler "not a symbol"))
          `(:quote ,symbol))
        (mu:nth 1 form)
       (mu:nthcdr 2 form))))

;;;
;;; defun compiles to a bound function symbol
;;;
(mu:intern core "defun-compiler"
   (:lambda (form env)
      ((:lambda (symbol lambda-form)
          (:if (mu:eq (mu:type-of symbol) :symbol)
               (:if (core:listp lambda-form)
                    (core:intern symbol (core:lambda-compiler lambda-form env))
                    (core:raise lambda-form 'core:defun-compiler "not a lambda list"))
               (core:raise symbol 'core:defun-compiler "not a symbol"))
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

(mu:intern core "if-compiler"
   (:lambda (form env)
      ((:lambda (len)                
          (:if (core:%orf (mu:eq len 3) (mu:eq len 4))
              ((:lambda (t-arm f-arm)           
                  (mu:compile `(:if ,(mu:cons (core:%thunk (mu:nth 1 form) env) ()) ,t-arm ,f-arm)))
               `(core:apply ,(core:%thunk (mu:nth 2 form) env) ())
               `(core:apply ,(core:%thunk (mu:nth 3 form) env) ()))
              (core:raise form 'core:if-compiler "takes two or three arguments")))
       (mu:length form))))

;;;
;;; quote
;;;
(mu:intern core "quote-compiler" (:lambda (form env) form))

;;;
;;; compiler
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
                             (core:%core-special-table fn))
                         (core:%compile-funcall fn args env))))
               (mu:car form)
               (mu:cdr form))
            form)))

(mu:intern core "compile"
   (:lambda (form)
     (core:%compile form ())))
