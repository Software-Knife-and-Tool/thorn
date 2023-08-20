;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; closures
;;;

;;;
;;; [*closure descriptor*] #s(:<ctype> :closure (lambda nargs fn env))
;;;
;;; lambda: lambda struct
;;; nargs:  number of reqs plus rest
;;; fn:     implementation
;;; env:    dynamic environment, list of frames
;;;
;;; closure
;;;
(mu:intern core :extern "closurep"
  (:lambda (fn)
     (mu:eq (core:type-of fn) 'closure)))

(mu:intern core :intern "make-closure"
   (:lambda (lambda fn env)
      ((:lambda (nargs)
          (core::make-core-type "closure"
            (core::pair-list
             `(:lambda ,lambda
               :nargs  ,nargs
               :fn     ,fn
               :env    ,env))))
       (mu:fx-sub
        (mu:length (core::lambda-prop :bound lambda))
        (:if (core::lambda-prop :rest lambda) 1 0)))))

(mu:intern core :intern "closure-prop"
   (:lambda (prop closure)
      (core:raise-unless core:closurep closure 'closure-prop "not a closure")
      (:if (core:findl (:lambda (closure-prop) (mu:eq prop closure-prop)) '(:lambda :nargs :fn :env))
           ((:lambda (ref)
               (core:raise-when core:null prop 'closure-prop "property not bound")
               (mu:cdr ref))
            (core::core-type-ref prop closure))
           (core::lambda-prop prop (core::closure-prop :lambda closure)))))

;;;
;;; create closed environment => env
;;;
(mu:intern core :intern "closure-env"
   (:lambda (lambda forms env)
      ((:lambda (bound-symbols)
          (core:foldl
           (:lambda (form acc)
              (:if (core:consp form)
                   ((:lambda (fn)
                       (:if (core:consp fn)
                            `(,@(core::closure-env lambda fn env) ,@acc)
                            `(,@(core::closure-env lambda (mu:cdr form) env) ,@acc)))
                    (mu:car form))
                   (:if (mu:eq :symbol (mu:type-of form))
                        (:if (core:uninternedp form)
                             ((:lambda (symbol)
                                 (:if symbol
                                      acc
                                      ((:lambda (binding-lambda)
                                          (:if binding-lambda
                                                (mu:cons (mu:cons form binding-lambda) acc)  
                                                acc))
                                       (core:findl
                                        (:lambda (lambda)
                                           (core:findl
                                            (:lambda (symbol) (mu:eq symbol form))
                                            (core::core-type-ref :bound lambda)))
                                        env))))
                                 (core:findl (:lambda (symbol) (mu:eq symbol form)) bound-symbols))
                             acc)
                        acc)))
           ()
           forms))
       (core::core-type-ref :bound lambda))))

;;;
;;; close-env = > closure
;;;
;;; mu::frames => list of [*frame descriptor*] (function . vector)
;;; env => list of (symbol . lambda)
;;;
(mu:intern core :intern "close-env"
  (:lambda (closure env)
     (core:raise-unless core:closurep closure 'core::close-env "closing bare function")
     ((:lambda (lambda fn env)
         (core::make-closure lambda fn env))
      (core::make-lambda
       (core::lambda-prop :bound fn)
       (core::lambda-prop :rest fn)
       (core:foldl
        (:lambda (frame))
        (core::closure-prop :env closure)
       (mu::frames))
      (core::closure-prop :fn fn)))))

;;;
;;; compile closed lambda => closure
;;;
(mu:intern core :intern "compile-closure"
   (:lambda (form env)
      ((:lambda (lambda)
          ((:lambda (fn)
              (core::make-closure lambda fn ()))
           (mu:compile
            `(:lambda
              ,(core::lambda-prop :bound lambda)
              ,(core::compile-lambda-body lambda (mu:nthcdr 2 form) env)))))
       (core::core-lambda (mu:nth 1 form) env))))

;;;
;;; apply closure to list of args => obj
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
