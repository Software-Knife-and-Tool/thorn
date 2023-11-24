;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; closures
;;;
;;;
;;; predicates
;;;
(mu:intern :core "functp"
   (:lambda (fn)
      (mu:eq 'funct (core:type-of fn))))

(mu:intern :core "functionp"
   (:lambda (function)
      (:if (mu:eq :func (mu:type-of function))
           :t
           (mu:eq 'funct (core:type-of function)))))

;;;
;;; [*function descriptor*] #s(:<ctype> :funct (lambda narg-list fn env))
;;;
;;; lambda: lambda ctype
;;; nargs:  number of reqs plus rest
;;; fn:     implementation
;;; env:    dynamic environment, list of frames
;;;
;;; function
;;;
(mu:intern :core "%make-funct"
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

(mu:intern :core "%funct-prop"
   (:lambda (prop funct)
      (:if (core:findl (:lambda (funct-prop) (mu:eq prop funct-prop)) '(:lambda :nargs :fn :env))
           ((:lambda (ref)
               ;;; (core:raise-when core:null prop 'core:%function-prop "property not bound")
               (mu:cdr ref))
            (core:%core-type-ref prop funct))
           (core:%lambda-prop prop (core:%funct-prop :lambda funct)))))

;;;
;;; create closed environment
;;;
;;; retruns env
;;;
(mu:intern :core "%closure-env"
   (:lambda (lambda body env)
      ((:lambda (bound-symbols)
          (core:%foldl
           (:lambda (elt acc)
              (:if (core:consp elt)
                   ((:lambda (function-form)
                       (:if (core:consp function-form)
                            `(,@(core:%closure-env lambda function-form env) ,@acc)
                            `(,@(core:%closure-env lambda (mu:cdr elt) env) ,@acc)))
                    (mu:car elt))
                   (:if (mu:eq :symbol (mu:type-of elt))
                        (:if (core:uninternedp elt)
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
           body))
       (core:%core-type-ref :bound lambda))))

;;;
;;; %close-env = > closure
;;;
;;; mu:frames => list of [*frame descriptor*] (function . vector)
;;; env => list of (symbol . lambda)
;;;
(mu:intern :core "%close-env"
  (:lambda (closure env)
     ((:lambda (lambda fn env)
         (core:%make-funct lambda fn env))
      (core:%make-lambda
       (core:%lambda-prop :bound fn)
       (core:%lambda-prop :rest fn)
       (mu:frames))
      (core:%funct-prop :fn fn))))

;;;
;;; compile closed lambda => function
;;;
(mu:intern :core "%compile-function"
   (:lambda (form env)
      ((:lambda (lambda)
          ((:lambda (fn)
              (core:%make-funct lambda fn ()))
           (mu:compile
            `(:lambda
              ,(core:%lambda-prop :bound lambda)
              ,(core:%compile-lambda-body lambda (mu:nthcdr 2 form) env)))))
       (core:%compile-core-lambda (mu:nth 1 form) env))))
