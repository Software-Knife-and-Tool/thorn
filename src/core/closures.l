;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; closures
;;;

;;;
;;; [*closure descriptor*] #s(:ctype :closure lambda nreqs fn dyn-env)
;;;
;;; lambda: lambda struct
;;; nreqs:  number of reqs plus rest
;;; fn:     implementation
;;; env:    dynamic environment, list of frames
;;;
(mu:intern core :intern "make-closure"
   (:lambda (lambda fn env)
      (core::make-ctype "closure"
        (core::pair-list
         `(:lambda ,lambda
           :nreqs ,(mu:length (core::ctype-ref :syms lambda))
           :fn ,fn
           :env ,env)))))

(mu:intern core :intern "closure-prop"
   (:lambda (prop closure)
      (core:raise-unless core:closurep closure 'closure-prop "not a closure")                 
      (:if (mu:eq prop :nreqs)
           (core::ctype-ref :nreqs closure)
           (:if (mu:eq prop :fn)
                (core::ctype-ref :fn closure)
                (:if (mu:eq prop :lambda)
                     (core::ctype-ref :fn closure)
                     (:if (mu:eq prop :env)
                          (core::ctype-ref :env closure)
                          (core::ctype-ref prop (core::ctype-ref :lambda closure))))))))

(mu:intern core :intern "closure"
  (:lambda (fn env)
     ((:lambda (lambda fn)
         (core::make-closure lambda fn (mu::frames)))
      (core::ctype-ref :lambda fn))
      (core::ctype-ref :fn fn)))

;;;
;;;
;;; [*frame descriptor*] (function . vector)
;;;
;;;
(mu:intern core :intern "closure-apply"
   (:lambda (fn args)
      ((:lambda (env)
        (core:mapc mu::fr-push env)
        ((:lambda (value)
           (core:mapc (:lambda (frame) (mu::fr-pop (mu:car frame))) env)
           value)
         ((:lambda (mu-fn)
            (mu:apply mu-fn (mu:eval (core::lambda-arg-list fn args))))
            (core::closure-prop :fn fn))))
        (core::closure-prop :env fn))))
