;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; closures
;;;

;;;
;;; [*closure descriptor*] #s(:ctype :closure (lambda nreqs fn env))
;;;
;;; lambda: lambda struct
;;; nreqs:  number of reqs plus rest
;;; fn:     implementation
;;; env:    dynamic environment, list of frames
;;;
(mu:intern core :intern "make-closure"
   (:lambda (lambda fn env)
      ((:lambda (nreqs) 
          (core::make-ctype "closure"
            (core::pair-list
             `(:lambda ,lambda
               :nreqs  ,nreqs
               :fn     ,fn
               :env    ,env))))
       (mu:fx-sub
        (mu:length (core::lambda-prop :bound lambda))
        (:if (core::lambda-prop :rest lambda) 1 0)))))

(mu:intern core :intern "closure-prop"
   (:lambda (prop closure)
      (core:raise-unless core:closurep closure 'closure-prop "not a closure")
      (:if (core:findl (:lambda (closure-prop) (mu:eq prop closure-prop)) '(:lambda :nreqs :fn :env))
           ((:lambda (ref)
               (core:raise-when core:null prop 'closure-prop "property not bound")
               (mu:cdr ref))
            (core::ctype-ref prop closure))
           (core::lambda-prop prop (core::closure-prop :lambda closure)))))

(mu:intern core :intern "compile-closure"
  (:lambda (lambda-desc body env)
    (core:maplist
     (:lambda (body)
       ((:lambda (form)
          (:if (mu:cdr body)
               form
               (:if (core:consp (mu:car body))
                    (:if (mu:eq 'lambda (mu:car (mu:car body)))
                         ((:lambda (lambda fn)
                             (core::make-closure lambda fn (mu:nthcdr 2 (mu::frames))))
                          (mu:cdr (core::ctype-ref :lambda form))
                          (mu:cdr (core::ctype-ref :fn form)))
                         form)
                    form)))
        (core::compile (mu:car body) env)))
     body)))

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
