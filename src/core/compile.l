;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; compile.l: core compiler
;;;
;;; compiling an environment
;;;
;;;    core::compile-add-env lambda-desc env => list
;;;
;;; core::complle-add-env adds a lambda descriptor to a list of
;;; lambda descriptors.
;;;
;;; arguments:
;;;
;;;   lambda-desc : lambda descriptor
;;;   env         : lexical environment list
;;;
;;; returns: augmented list of lambda descriptors
;;;
(mu:intern core::ns :intern "compile-add-env"
  (:lambda (lambda-desc env)
     (core:raise-unless core:structp lambda-desc 'compile-add-env "not a lambda descriptor")
     (core:raise-unless core:listp env 'compile-add-env "not a list")
     (mu:cons lambda-desc env)))

;;;
;;; compiling special/macro forms
;;;
;;;    core::defconst form env => list
;;;    core::defmacro form env => list
;;;    core::defun form env => list
;;;    core::if form env => list
;;;    core::lambda form env => list
;;;    core::quote form env => list
;;;
;;; special/macro form compiler rewrites a form against
;;; an environment into something the runtime can execute.
;;;
;;; arguments:
;;;
;;;   form : any form
;;;   env  : lexical environment list
;;;
;;; returns: mu form
;;;
(mu:intern core::ns :intern "quote" (:lambda (form env) form))

;;;
;;; lambda compiles to a closure descriptor
;;;
(mu:intern core::ns :intern "lambda"
   (:lambda (form env)
    (core:raise-if (mu:fx-lt (mu:length form) 2) form 'lambda "malformed lambda expression")
    ((:lambda (lambda body)
        ((:lambda (lambda-desc)
            ((:lambda (fn)              
                (:if (core::or lambda-env restsym)
                     (core::make-closure lambda-desc fn)
                     fn))
             (mu:compile `(:lambda ,symbols ,@(core::compile-lambda-body lambda-desc body env)))))
        (core::core-lambda lambda env)))
     (mu:nth 1 form)
     (mu:nthcdr 2 form))))

(mu:intern core::ns :intern "defmacro"
   (:lambda (form env)
      ((:lambda (symbol lambda-form)
          (core:raise-unless core:symbolp symbol 'defmacro "not a symbol")
          ((:lambda (macrofn)
              (core::intern symbol macrofn))
           (core::core-macro-env symbol (core::compile-macro lambda-form env))))
       (mu:nth 1 form)
       (mu:nthcdr 2 form))))

;;; (mu:compile (core::list-2 :quote (core::intern symbol (core::lambda (mu:cons :lambda lambda) env) env)))

(mu:intern core::ns :intern "defun"
   (:lambda (form env)
      ((:lambda (symbol lambda-form)
          (core:raise-unless core:symbolp symbol 'defun "not a symbol")
          (core:raise-when core:keywordp symbol 'defun "not a symbol")
          (core:raise-unless core:listp lambda-form 'defun "not a lambda list")
          ;;; we get away with this only because core:lambda ignores the initial symbol
          (core::intern symbol (core::lambda lambda-form env)))
       (mu:nth 1 form)
       (mu:nthcdr 1 form))))

;;;
;;; this if form can take two or three arguments
;;;
(mu:intern core::ns :intern "thunk"
  (:lambda (form env)
    (core::compile
     `(:lambda () ,form)
     env)))

(mu:intern core::ns :intern "if-body"
  (:lambda (term true-fn false-fn)
    (:if term
         (core:apply true-fn ())
         (core:apply false-fn ()))))

(mu:intern core::ns :intern "if"
   (:lambda (form env)
      (core:raise-unless core:listp env 'if "not an environment")
      (core:raise-unless (:lambda (form) (mu:fx-lt 2 (mu:length form))) form 'if "takes two or three arguments")
      (core:raise-unless (:lambda (form) (mu:fx-lt (mu:length form) 5)) form 'if "takes two or three arguments")
      (mu:cons 'core::if-body
        `(,(mu:cons (core::thunk (mu:nth 1 form) env) ())
          ,(core::thunk (mu:nth 2 form) env)
          ,(core::thunk (mu:nth 3 form) env)))))

(mu:intern core::ns :intern "special-table"
   (:lambda (symbol)
      (mu:cdr (core:assoc
        symbol
        '((defun    . core::defun)
          (defmacro . core::defmacro)
          (if       . core::if)
          (lambda   . core::lambda)
          (quote    . core::quote))))))

;;;
;;; compile
;;;
(mu:intern core::ns :intern "compile"
   (:lambda (form env)
      (:if (core:consp form)
           ((:lambda (fn args)
               (:if (core::or (core:closurep fn) (core:functionp fn))
                    (core::compile-fn-call fn args env)
                    (:if (core:keywordp fn)
                         (mu:compile form)     ; mu special form (:key ...)
                         (:if (core:consp fn)  ; application ((...) ...), native or core
                              (core::compile-lambda-call fn args env)
                              (:if (core:symbolp fn) ; symbol, function symbol or special form
                                   ((:lambda (special)
                                       (:if special
                                            (mu:apply (mu:sy-val special) `(,form ,env))
                                            (:if (core:fboundp fn)
                                                 ((:lambda (macro-fn)
                                                     (:if macro-fn
                                                          (core::compile-macro-call fn args env)
                                                          (core::compile-symbol-call fn args env)))
                                                  (core:macro-function fn env))
                                                 (core::compile-symbol-call fn args env))))
                                    (core::special-table fn))
                                   (core:raise form 'core::compile "not a function designator"))))))
            (mu:car form)
            (mu:cdr form))
           (:if (core:symbolp form)
                (core::compile-symbol form env)
                form))))

(mu:intern core::ns :extern "compile"
   (:lambda (form)
     (core::compile form ())))
