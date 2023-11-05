;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; compile applications
;;;

;;;
;;; core argument lists
;;;
(mu:intern :core "%arg-list"
    (:lambda (arg-list)
      (core:foldr
       (:lambda (elt acc)
         `(mu:cons ,elt ,acc))
       ()
       arg-list)))

(mu:intern :core "%lambda-arg-list"
   (:lambda (function arg-list)
      (:if (core:functp function)
           ((:lambda (rest nreqs)
               (:if (core:%andf rest (core:zerop nreqs))
                    `(mu:cons ,(core:%arg-list arg-list) ())
                    ((:lambda (reqs rest)
                        (core:%arg-list `(,@reqs ,(core:%arg-list rest))))
                     (core:dropr arg-list (mu:fx-sub (mu:length arg-list) nreqs))
                     (core:dropl arg-list nreqs))))
            (core:%funct-prop :rest function)
            (core:%funct-prop :nargs function))
           (core:%arg-list arg-list))))

;;;
;;; compiled argument lists
;;;
(mu:intern :core "%compile-arg-list"
   (:lambda (arg-list env)
      (core:foldr
       (:lambda (elt acc)
          (mu:cons 'mu:cons (mu:cons elt `(,acc))))
       ()
       (core:mapcar
        (:lambda (form)
           (core:%compile form env))
        arg-list))))

(mu:intern :core "%compile-lambda-arg-list"
   (:lambda (function arg-list env)
     (core:%lambda-arg-list function (core:%mapcar (:lambda (elt) (core:%compile elt env)) arg-list))))

;;;
;;; compile-funcall
;;;
;;; compile function applications to mu-appliable forms
;;;
(mu:intern :core "%compile-lambda-call"
   (:lambda (lambda-form arg-list env)
      ((:lambda (compiled-function)
        (:if (core:functionp compiled-function)
             (:if (core:functp compiled-function)
                  `(core:%funct-apply-quoted ,compiled-function ,(core:%compile-lambda-arg-list compiled-function arg-list env))
                  `(mu:apply ,compiled-function ,(core:%compile-arg-list arg-list env)))
             (core:raise compiled-function 'core:%compile-lambda-call "illegal function call")))
     (core:%compile lambda-form env))))

(mu:intern :core "%compile-symbol-call"
   (:lambda (function-symbol arg-list env)
      (:if (core:boundp function-symbol)
           ((:lambda (function)
               (:if (core:functionp function)
                    (:if (core:functp function)
                         `(core:%funct-apply-quoted ,fn ,(core:%compile-lambda-arg-list function arg-list env))
                         `(mu:apply ,function ,(core:%compile-arg-list arg-list env)))
                    (core:raise function 'core:%compile-symbol-call "illegal function call")))
            (core:symbol-value function-symbol))
           `(core:apply ,function-symbol ,(core:%compile-arg-list arg-list env)))))

(mu:intern :core "%compile-funcall"
   (:lambda (function-form arg-list env)
      (:if (core:consp function-form)
           (core:%compile-lambda-call function-form arg-list env)
           (:if (mu:eq :symbol (mu:type-of function-form))
                (core:%compile-symbol-call function-form arg-list env)
                (core:raise function-form 'core:%compile-funcall "illegal function call")))))

;;;
;;; apply closure to argument list
;;;
(mu:intern :core "%funct-apply"
   (:lambda (funct arg-list)
      ((:lambda (env)
        (core:%mapc mu:fr-push env)
        ((:lambda (value)
           (core:%mapc (:lambda (frame) (mu:fr-pop (mu:car frame))) env)
           value)
         ((:lambda (mu-fn)
             (mu:apply mu-fn (mu:eval (core:%lambda-arg-list funct arg-list))))
          (core:%funct-prop :fn funct))))
        (core:%funct-prop :env funct))))

(mu:intern :core "%funct-apply-quoted"
   (:lambda (funct arg-list)
      ((:lambda (env)
        (core:%mapc mu:fr-push env)
        ((:lambda (value)
           (core:%mapc (:lambda (frame) (mu:fr-pop (mu:car frame))) env)
           value)
         ((:lambda (mu-fn)
             (mu:apply mu-fn arg-list))
          (core:%funct-prop :fn funct))))
        (core:%funct-prop :env funct))))
