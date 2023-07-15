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
           ((:lambda (symbols reqsyms restsym lambda-env closed)
             ((:lambda (form)                
                 (:if (core::or lambda-env restsym)           
                      (core::make-closure lambda-env (mu:length reqsyms) restsym form)
                      form))
              (mu:compile `(:lambda ,symbols ,@(core::compile-lambda-body lambda-desc body env)))))
           (core::lambda-property :syms lambda-desc)
           (core::lambda-property :reqs lambda-desc)
           (core::lambda-property :rest lambda-desc)
           (core::lambda-property :env lambda-desc)
           (core::lambda-property :closed lambda-desc)))
       (core::core-lambda lambda env)))
     (mu:nth 1 form)
     (mu:nthcdr 2 form))))

(mu:intern core::ns :intern "defconst"
   (:lambda (form env)
      ((:lambda (symbol value)
          (:if (mu:eq :symbol (mu:type-of symbol))
               (mu:compile
                `(:quote
                  ,(core::intern
                    symbol
                    (mu:eval (core::compile value env)))))
               (core:raise symbol 'defconst "not a symbol")))
          (mu:nth 1 form)
          (mu:nth 2 form))))

(mu:intern core::ns :intern "defmacro"
   (:lambda (form env)
      ((:lambda (symbol lambda)
          (core:raise-unless core:listp lambda 'defmacro "not a lambda list")
          (:if (mu:eq :symbol (mu:type-of symbol))
               ((:lambda (macro-fn)
                   ((:lambda (symbol)
                       (core::core-macro-env symbol macro-fn)
                       (mu:compile `(:quote ,symbol)))
                    (core::intern symbol macro-fn)))
                (core::compile-macro `(,(mu:car lambda) ,(mu:cdr lambda)) env))
               (core:raise symbol 'defmacro "not a symbol")))
       (mu:nth 1 form)
       (mu:nthcdr 2 form))))

(mu:intern core::ns :intern "defctype"
   (:lambda (form env)
      ((:lambda (symbol slot-defs)
         (core:raise-unless core:listp slot-defs 'defctype "not a slot-defs list")
         (:if (mu:eq :symbol (mu:type-of symbol))
              ((:lambda (pairs)
                (core::def-ctype (mu:sy-name symbol) pairs)
                (mu:compile `(:quote ,symbol)))
               (core::pair-list slot-defs))
              (core:raise symbol 'defctype "not a symbol")))
       (mu:nth 1 form)
       (mu:nthcdr 2 form))))

(mu:intern core::ns :intern "defun"
   (:lambda (form env)
      ((:lambda (symbol lambda body)
          (core:raise-unless core:symbolp symbol 'defun "not a symbol")
          (core:raise-unless core:listp lambda 'defun "not a lambda list")
          ((:lambda (lambda-desc)
              ((:lambda (compiled-lambda compiled-body)               
                  (mu:cons :quote
                     (mu:cons (core::intern symbol (mu:compile `(:lambda ,compiled-lambda ,@compiled-body)))
                        ())))
               (core::lambda-property :syms lambda-desc)
               (core::compile-lambda-body lambda-desc body env)))
           (core::core-lambda lambda env)))
       (mu:nth 1 form)
       (mu:nth 2 form)
       (mu:nthcdr 3 form))))

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
        '((defconst . core::defconst)
          (defun    . core::defun)
          (defmacro . core::defmacro)
          (defctype . core::defctype)
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
             (:if (core:keywordp fn)
                  (mu:compile form)     ;;; mu special form (:key ...)
                  (:if (core:consp fn)  ;;; application ((...) ...), native or core
                       (core::compile-application fn args env)
                       ((:lambda (special)
                          (:if special
                               (mu:apply (mu:sy-val special) `(,form ,env))
                               (core::compile-application fn args env)))
                        (core::special-table fn)))))
           (mu:car form)
           (mu:cdr form))
          (:if (mu:eq :symbol (mu:type-of form))
               (core::compile-symbol form env)
               (mu:compile form)))))

(mu:intern core::ns :extern "compile"
   (:lambda (form)
     (core::compile form ())))
