;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; compile applys
;;;

;;;
;;; core argument lists
;;;
(mu:intern core "%arg-list"
    (:lambda (args)
      (core:foldr
       (:lambda (elt acc)
         `(mu:cons ,elt ,acc))
       ()
       args)))

(mu:intern core "%lambda-arg-list"
   (:lambda (fn args)
      (:if (core:functionp fn)
           ((:lambda (rest nreqs)
               (:if (core:%andf rest (core:zerop nreqs))
                    `(mu:cons ,(core:%arg-list args) ())
                    ((:lambda (reqs rest)
                        (core:%arg-list `(,@reqs ,(core:%arg-list rest))))
                     (core:dropr args (mu:fx-sub (mu:length args) nreqs))
                     (core:dropl args nreqs))))
            (core:%function-prop :rest fn)
            (core:%function-prop :nargs fn))
           (core:%arg-list args))))

(mu:intern core "%quoted-lambda-arg-list"
  (:lambda (fn args)
     (:if (core:functionp fn)
         args
         ((:lambda (rest nreqs)
             (:if rest
                  (:if (core:%andf rest (core:zerop nreqs))
                       `(,args)
                       `(,@(core:dropr args (mu:fx-sub (mu:length args) nreqs))
                         ,@(core:dropl args nreqs)))
                  args))
            (core:%function-prop :rest fn)
            (core:%function-prop :nargs fn)))))

;;;
;;; compiled argument lists
;;;
(mu:intern core "%compile-arg-list"
   (:lambda (args env)
      (core:foldr
       (:lambda (elt acc)
          (mu:cons 'mu:cons (mu:cons elt `(,acc))))
       ()
       (core:mapcar
        (:lambda (form)
           (core:%compile form env))
        args))))

(mu:intern core "%compile-quoted-lambda-arg-list"
   (:lambda (fn args env)
     (core:%quoted-lambda-arg-list
      fn
      (core:%mapcar (:lambda (elt) (core:%compile elt env)) args))))

(mu:intern core "%compile-lambda-arg-list"
   (:lambda (fn args env)
     (core:%lambda-arg-list
      fn
      (core:%mapcar (:lambda (elt) (core:%compile elt env)) args))))

;;;
;;; compile-application
;;;
;;; expand macros
;;; convert core lambdas to mu forms
;;; compile function applications
;;;
(mu:intern core "%compile-lambda-call"
   (:lambda (form args env)
     ((:lambda (fn)
        (:if (core:core-function-p fn)
             `(mu:apply ,(core:%function-prop :fn fn) ,(core:%compile-lambda-arg-list fn args env))
             `(mu:apply ,fn ,(core:%compile-arg-list args env))))
     (core:%compile form env))))

(mu:intern core "%compile-fn-call"
   (:lambda (fn args env)
      (:if (core:core-function-p fn)
           `(core:apply ,fn ,(core:%compile-quoted-lambda-arg-list fn args env))
           `(mu:apply ,fn ,(core:%compile-arg-list args env)))))

(mu:intern core "%compile-symbol-call"
   (:lambda (symbol args env)
     (:if (core:boundp symbol)
          ((:lambda (fn)
              (:if (core:functionp fn)
                   (:if (core:core-function-p fn)
                        `(core:%function-apply ,fn ,(core:%compile-lambda-arg-list fn args env))
                        `(mu:apply ,fn ,(core:%compile-arg-list args env)))
                   (core:raise fn 'core:%compile-symbol-call "not appliable")))
           (core:symbol-value symbol))
          `(,core:apply ,symbol ,(core:%compile-arg-list args env)))))

(mu:intern core "%compile-funcall"
   (:lambda (operator args env)
      (core:%compile-fn-call operator args env)
      (:if (core:consp operator)
           (core:%compile-lambda-call operator args env)
           (:if (mu:eq :symbol (mu:type-of operator))
                (core:%compile-symbol-call operator args env)
                (core:raise form 'core:%compile "illegal function call")))))
