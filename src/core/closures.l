;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; closures
;;;

;;;
;;; [*closure descriptor*] #s(:ctype :closure nreqs fn #s(:ctype :lambda syms reqs rest env))
;;;
;;; nreqs: number of required arguments
;;; fn:    implementation
;;;
;;; lambda:
;;; env:   list of frame state structs
;;; reqs:  required symbols
;;; rest:  rest symbol or nil
;;; syms:  all symbols
;;;
(mu:intern core::ns :intern "make-closure"
   (:lambda (lambda fn)
      (core::make-ctype "closure"
        (core::pair-list
         `(:lambda ,lambda
           :nreqs ,(mu:length (core::ctype-ref :reqs lambda))
           :fn ,fn)))))

(mu:intern core::ns :intern "closure-prop"
   (:lambda (prop closure)
      (core:raise-unless core:closurep closure 'closure-prop `(,prop ,(mu:type-of closure) "not a closure"))                 
      (:if (mu:eq prop :fn)
           (core::ctype-ref :fn closure)
           (:if (mu:eq prop :nreqs)
                (core::ctype-ref :nreqs closure)
                (core::ctype-ref prop (core::ctype-ref :lambda closure))))))

(mu:intern core::ns :intern "closure"
  (:lambda (fn env)
     (core:raise-unless core:closurep fn 'core::closure "closing bare function")
     ((:lambda (lambda fn)
         (core::make-closure lambda fn))
      (core::make-lambda
       (core::ctype-ref :syms fn)
       (core::ctype-ref :reqs fn)
       (core::ctype-ref :rest fn)
       env)
      (core::ctype-ref :fn fn))))

(mu:intern core::ns :intern "closure-apply"
   (:lambda (fn args)
      ((:lambda (env)
        (core:mapc mu:fr-push env)
        ((:lambda (value)
           (core:mapc mu:fr-pop env)
           value)
         ((:lambda (mu-fn)
            (mu:apply mu-fn (mu:eval (core::lambda-arg-list fn args))))
            (core::closure-prop :fn fn))))
        (core::closure-prop :env fn))))
