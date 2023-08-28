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

;;;
;;; lambda compiles to a closure core-type
;;;
(mu:intern core "lambda-compiler"
    (:lambda (form env)
    (core:raise-if (mu:fx-lt (mu:length form) 2) form 'lambda "malformed lambda expression")
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
     (mu:nthcdr 2 form))))

;;;
;;; defmacro compiles to a bound macro symbol
;;;
(mu:intern core "defmacro-compiler"
   (:lambda (form env)
      ((:lambda (symbol lambda-form)
          (core:raise-unless core:symbolp symbol 'core:defmacro-compiler "not a symbol")
          ((:lambda (macrofn)
              (core:intern symbol macrofn))
           (core:%core-macro-env symbol (core:%compile-macro lambda-form env))))
       (mu:nth 1 form)
       (mu:nthcdr 2 form))))

;;;
;;; defun compiles to a bound function symbol
;;;
(mu:intern core "defun-compiler"
   (:lambda (form env)
      ((:lambda (symbol lambda-form)
          (core:raise-unless core:symbolp symbol 'defun "not a symbol")
          (core:raise-when core:keywordp symbol 'defun "not a symbol")
          (core:raise-unless core:listp lambda-form 'defun "not a lambda list")
          ;;; we get away with this only because core:lambda ignores the initial symbol
          (core:intern symbol (core:lambda-compiler lambda-form env)))
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

(mu:intern core "%if-body"
  (:lambda (term true-fn false-fn)
    (:if term
         (core:apply true-fn ())
         (core:apply false-fn ()))))

(mu:intern core "if-compiler"
   (:lambda (form env)
      (core:raise-unless core:listp env 'if "not an environment")
      (core:raise-unless (:lambda (form) (mu:fx-lt 2 (mu:length form))) form 'if "takes two or three arguments")
      (core:raise-unless (:lambda (form) (mu:fx-lt (mu:length form) 5)) form 'if "takes two or three arguments")
      (mu:cons 'core:%if-body
        `(,(mu:cons (core:%thunk (mu:nth 1 form) env) ())
          ,(core:%thunk (mu:nth 2 form) env)
          ,(core:%thunk (mu:nth 3 form) env)))))

;;;
;;; quote
;;;
(mu:intern core "quote-compiler" (:lambda (form env) form))

;;;
;;; compiler
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

(mu:intern core "%compile"
   (:lambda (form env)
      (:if (core:consp form)
           ((:lambda (fn args)
               (:if (core:%appliablep fn)
                    (core:%compile-fn-call fn args env)
                    (:if (core:keywordp fn)
                         (mu:compile form)     ; mu special form (:key ...)
                         (:if (core:consp fn)  ; application ((...) ...), native or core
                              (core:%compile-lambda-call fn args env)
                              (:if (mu:eq :symbol (mu:type-of fn)) ; symbol, function symbol or special form
                                   ((:lambda (special)
                                       (:if special
                                            (mu:apply (mu:sy-val special) `(,form ,env))
                                            (:if (core:fboundp fn)
                                                 ((:lambda (macro-fn)
                                                     (:if macro-fn
                                                          (core:%compile-macro-call fn args env)
                                                          (core:%compile-symbol-call fn args env)))
                                                  (core:macro-function fn env))
                                                 (core:%compile-symbol-call fn args env))))
                                    (core:%core-special-table fn))
                                   (core:raise form 'core:%compile "not a function designator"))))))
            (mu:car form)
            (mu:cdr form))
           form)))

(mu:intern core "compile"
   (:lambda (form)
     (core:%compile form ())))
