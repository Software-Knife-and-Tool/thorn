;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; compile funcalls
;;;

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
           ((:lambda (rest nreqs)
               (:if (core::and rest (core:zerop nreqs))
                    `(mu:cons ,(core::flat-arg-list args) ())
                    ((:lambda (reqs rest)
                        (core::flat-arg-list `(,@reqs ,(core::flat-arg-list rest))))
                     (core:dropr args (mu:fx-sub (mu:length args) nreqs))
                     (core:dropl args nreqs))))
            (core::closure-prop :rest fn)
            (core::closure-prop :nreqs fn))
           (core::flat-arg-list args))))

(mu:intern core::ns :intern "quoted-lambda-arg-list"
  (:lambda (fn args)
     (:if (core::appliablep fn)
         args
         ((:lambda (rest nreqs)
             (:if rest
                  (:if (core::and rest (core:zerop nreqs))
                       `(,args)
                       `(,@(core:dropr args (mu:fx-sub (mu:length args) nreqs))
                         ,@(core:dropl args nreqs)))
                  args))
            (core::closure-prop :rest fn)
            (core::closure-prop :nreqs fn)))))

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
         `(,fn ,(core::compile-quoted-lambda-arg-list fn args env)))))

(mu:intern core::ns :intern "compile-macro-call"
  (:lambda (macro-symbol args env)
    (core::compile
     (core:macroexpand `(macro-symbol ,@args) env)
     env)))

(mu:intern core::ns :intern "compile-symbol-call"
  (:lambda (symbol args env)
     (:if (core:boundp symbol)
        ((:lambda (fn)
             (core:raise-unless core::appliablep fn 'core:compile-symbol-call "not a function designator")
             (:if (core:closurep fn)
                  `(core:apply ,(core::closure-prop :fn fn) ,(core::compile-quoted-lambda-arg-list fn args env))
                  `(core:apply ,fn ,(core::compile-flat-arg-list args env))))
          (core:symbol-value symbol))
         `(core:apply ,symbol ,(core::compile-flat-arg-list args env)))))
