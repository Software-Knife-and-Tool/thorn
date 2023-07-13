;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; [*closure descriptor*] #s(:closure env nreqs rest func)
;;;
;;; env:   list of frame state structs
;;; nreqs: number of required arguments
;;; rest:  rest symbol or nil
;;; func:  function, mu applyable
;;;
(mu:intern core::ns :intern "closure-property"
  (:lambda (key closure)
    (core:raise-unless core:closurep closure 'core::closure-property "is not a closure")
    (mu:sv-ref
     (mu:st-vec closure)
     (mu:cdr
      (core:assoc
       key
       '((:env   . 0)
         (:nreqs . 1)
         (:rest  . 2)
         (:func  . 3)))))))

(mu:intern core::ns :intern "make-closure"
  (:lambda (env nreqs rest func)
    (mu:make-st :closure
      `(,env ,nreqs ,rest ,func))))

;;;
;;; create a closure
;;;
(mu:intern core::ns :intern "closure"
  (:lambda (fn)
    ((:lambda (lambda-desc)
       (core:raise-when core:null lambda-desc 'core::closure "closing bare function")
       ((:lambda (desc body)
          (mu:compile `(:lambda ,(core::lambda-property :syms desc) ,desc ,@(mu:cdr (core::fn-form fn)))))
        (core::make-lambda
         (core::lambda-property :syms lambda-desc)
         (core::lambda-property :reqs lambda-desc)
         (core::lambda-property :rest lambda-desc)
         (core::lambda-property :env lambda-desc)
         (core:mapcar
          (:lambda (desc)
             (mu:fr-get (core::lambda-property :frame desc)))
          (core::lambda-property :env lambda-desc)))
        (mu:cdr (core::fn-form fn))))
     (core::fn-lambda-desc fn))))

;;;
;;; core argument lists
;;;
(mu:intern core::ns :intern "flat-arg-list"
    (:lambda (args)
      (core:foldr
       (:lambda (elt acc)
         `(mu:cons ,elt ,acc))
       ()
       args)))

(mu:intern core::ns :intern "lambda-arg-list"
   (:lambda (fn args)
      (:if (core:closurep fn)
           ((:lambda (nreqs)
               (:if (core::and (core::closure-property :rest fn) (core:zerop nreqs))
                    `(mu:cons ,(core::flat-arg-list args) ())
                    ((:lambda (reqs rest)
                              (core::flat-arg-list `(,@reqs ,(core::flat-arg-list rest))))
                     (core:dropr args (mu:fx-sub (mu:length args) nreqs))
                     (core:dropl args nreqs))))
            (core::closure-property :nreqs fn))
           (core::flat-arg-list args))))

(mu:intern core::ns :intern "quoted-lambda-arg-list"
  (:lambda (fn args)
    (:if (core:functionp fn)
         args
         (:if (core::closure-property :rest fn)
              (:if (core::and (core:closure-property :rest fn) (core:zerop (core::closure-property :arity fn)))
                   `(,args)
                   `(,@(core:dropr args (mu:fx-sub (mu:length args) (core::closure-property :arity fn)))
                     ,@(core:dropl args (core::closure-property :arity fn))))
              args))))

;;;
;;; compiled argument lists
;;;
(mu:intern core::ns :intern "compile-flat-arg-list"
  (:lambda (args env)
    (core:foldr
     (:lambda (elt acc)
       (mu:cons 'mu:cons (mu:cons (core::compile elt env) `(,acc))))
     ()
     args)))

(mu:intern core::ns :intern "compile-quoted-lambda-arg-list"
   (:lambda (fn args env)
     (core::quoted-lambda-arg-list
      fn
      (core:mapcar (:lambda (elt) (core::compile elt env)) args))))

(mu:intern core::ns :intern "closure-apply"
   (:lambda (fn args)
    ((:lambda (env)
        (core:mapc mu:fr-push env)
        ((:lambda (value)
           (core:mapc mu:fr-pop env)
           value)
         ((:lambda (mu-fn)
            (mu:apply mu-fn (mu:eval (core::lambda-arg-list fn args))))
            (core::closure-property :func fn))))
        (core::closure-property :env fn))))

;;;
;;; compile-application
;;;
;;; expand macros
;;; convert core lambdas to mu forms
;;; compile function applications
;;;
(mu:intern core::ns :intern "compile-lambda-call"
  (:lambda (form args env)
    ((:lambda (fn)
       (:if (core:closurep fn)
            `(core::closure-apply ,fn ,(core::compile-flat-arg-list args env))
            (mu:cons fn (core::compile-quoted-lambda-arg-list fn args env))))
     (core::compile form env))))

(mu:intern core::ns :intern "compile-fn-call"
  (:lambda (fn args env)
    (:if (core:closurep fn)
         `(core:apply ,fn ,(core::compile-flat-arg-list args env))
         (mu:cons fn (core::compile-quoted-lambda-arg-list fn args env)))))

(mu:intern core::ns :intern "compile-macro-call"
  (:lambda (macro-symbol args env)
    (core::compile
     (core:macroexpand `(macro-symbol ,@args) env)
     env)))

(mu:intern core::ns :intern "compile-symbol-call"
  (:lambda (symbol args env)
    (:if (core:boundp symbol)
         ((:lambda (fn)
            (:if (core:closurep fn)
                 (:if (core::lambda-property :desc fn)
                      `(core:apply ,fn ,(core::compile-flat-arg-list args env))
                      `(,fn ,@(core::compile-quoted-lambda-arg-list fn args env)))
                 `(mu:apply ,fn ,(core::compile-flat-arg-list args env))))
          (core:symbol-value (core::compile symbol env)))
         `(core:apply ,(core::compile symbol env) ,(core::compile-flat-arg-list args env)))))

;;;
;;; compile function application
;;;
(mu:intern core::ns :intern "compile-application"
  (:lambda (fn args env)
    (:if (core:functionp fn)
         (core::compile-fn-call fn args env)
         (:if (core:consp fn)
              (core::compile-lambda-call fn args env)
              (:if (mu:eq :symbol (mu:type-of fn))
                   (:if (core:boundp fn)
                        ((:lambda (macro-fn)
                           (:if macro-fn
                                (core::compile-macro-call fn args env)
                                (core::compile-symbol-call fn args env)))
                         (core:macro-function fn env))
                        (core::compile-symbol-call fn args env))
                   (core:raise fn 'compile-application "not a function designator"))))))
