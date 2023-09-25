;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; closures
;;;

;;;
;;; [*function descriptor*] #s(:<ctype> :funct (lambda nargs fn env))
;;;
;;; lambda: lambda struct
;;; nargs:  number of reqs plus rest
;;; fn:     implementation
;;; env:    dynamic environment, list of frames
;;;
;;; function
;;;
(mu:intern core "%make-function"
   (:lambda (lambda fn env)
      ((:lambda (nargs)
          (core:%make-core-type "funct"
            (core:%pair-list
             `(:lambda ,lambda
               :nargs  ,nargs
               :fn     ,fn
               :env    ,env))))
       (mu:fx-sub
        (mu:length (core:%lambda-prop :bound lambda))
        (:if (core:%lambda-prop :rest lambda) 1 0)))))

(mu:intern core "%function-prop"
   (:lambda (prop fn)
      (:if (core:findl (:lambda (fn-prop) (mu:eq prop fn-prop)) '(:lambda :nargs :fn :env))
           ((:lambda (ref)
               ;;; (core:raise-when core:null prop 'core:%function-prop "property not bound")
               (mu:cdr ref))
            (core:%core-type-ref prop fn))
           (core:%lambda-prop prop (core:%function-prop :lambda fn)))))

;;;
;;; create closed environment => env
;;;
(mu:intern core "%closure-env"
   (:lambda (lambda forms env)
      ((:lambda (bound-symbols)
          (core:%foldl
           (:lambda (form acc)
              (:if (core:consp form)
                   ((:lambda (fn)
                       (:if (core:consp fn)
                            `(,@(core:%closure-env lambda fn env) ,@acc)
                            `(,@(core:%closure-env lambda (mu:cdr form) env) ,@acc)))
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
                                            (core:%core-type-ref :bound lambda)))
                                        env))))
                                 (core:findl (:lambda (symbol) (mu:eq symbol form)) bound-symbols))
                             acc)
                        acc)))
           ()
           forms))
       (core:%core-type-ref :bound lambda))))

;;;
;;; %close-env = > closure
;;;
;;; mu:frames => list of [*frame descriptor*] (function . vector)
;;; env => list of (symbol . lambda)
;;;
(mu:intern core "%close-env"
  (:lambda (closure env)
     ((:lambda (lambda fn env)
         (core:%make-function lambda fn env))
      (core:%make-lambda
       (core:%lambda-prop :bound fn)
       (core:%lambda-prop :rest fn)
       (mu:frames))
      (core:%function-prop :fn fn))))

;;;
;;; compile closed lambda => function
;;;
(mu:intern core "%compile-function"
   (:lambda (form env)
      ((:lambda (lambda)
          ((:lambda (fn)
              (core:%make-function lambda fn ()))
           (mu:compile
            `(:lambda
              ,(core:%lambda-prop :bound lambda)
              ,(core:%compile-lambda-body lambda (mu:nthcdr 2 form) env)))))
       (core:%core-lambda (mu:nth 1 form) env))))
