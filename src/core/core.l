;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core namespace
;;;
(mu:make-ns :core)

;;;
;;; core types
;;;
(mu:intern :core "%init-core-ns"
   (:lambda ()
      ;;; core function
      (core:define-core-type "funct"
        '((:lambda  . :struct)
          (:nargs   . :fixnum)
          (:fn      . :func)
          (:env     . :listp)))

      ;;; core lambda
      (core:define-core-type "lambda"
        '((:bound  . :listp)
          (:rest   . :symbolp)
          (:lexenv . :listp)))

      ;;; core exception
      (core:define-core-type "except"
        '((:cond   . :symbolp)
          (:object . :t)
          (:source . :symbolp)
          (:reason . :string)
          (:env    . :listp)))

      ;;; core package
      (core:define-core-type "package"
        '((:name    . :string)
          (:intern  . :keyword)
          (:extern  . :keyword)
          (:ch-mac  . :keyword)
          (:sy-mac  . :keyword)
          (:uses    . :listp)))

      ;;; core vector
      (core:define-core-type "vector"
        '((:base    . :vector)
          (:start   . :fixnum)
          (:length  . :fixnum)))
      
      (mu:intern :core "%packages%" (core:%gen-anon-ns))
      (mu:intern :core "%char-macro-ns%" (core:%gen-anon-ns))
      (mu:intern :core "%macro-ns%" (core:%gen-anon-ns))
      (mu:intern :core "%symbol-macro-ns%" (core:%gen-anon-ns))))

;;;
;;; predicates
;;;
(mu:intern :core "not" (:lambda (t) (core:null t)))
(mu:intern :core "atom" (:lambda (obj) (core:not (core:consp obj))))
(mu:intern :core "constantp"
   (:lambda (form)
      (:if (core:%orf (core:null form) (core:keywordp form))
           :t
           (:if (core:consp form)
                (mu:eq :quote (mu:car form))
                (core:not (core:symbolp form))))))
           
(mu:intern :core "numberp"
  (:lambda (n)
     (:if (core:fixnump n)
          :t
          (core:floatp n))))

(mu:intern :core "zerop"
   (:lambda (n)
     (:if (core:numberp n)
          (:if (mu:eq 0 n)
               :t
               (mu:eq 0.0 n))
          (core:raise n 'core:zerop "not a number"))))

(mu:intern :core "minusp"
   (:lambda (n)
      (:if (core:numberp n)
           (:if (core:floatp n)
                (mu:fl-lt n 0.0)
                (mu:fx-lt n 0))
           (core:raise n 'core:minusp "not a number"))))

(mu:intern :core "plusp"
   (:lambda (n)
      (:if (core:numberp n)
           (:if (core:floatp n)
                (mu:fl-lt 0.0 n)
                (mu:fx-lt 0 n))
           (core:raise n 'core:plusp "not a number"))))

;;;
;;; system classes
;;;
(mu:intern :core "charp" (:lambda (t) (mu:eq :char (mu:type-of t))))
(mu:intern :core "fixnump" (:lambda (t) (mu:eq :fixnum (mu:type-of t))))
(mu:intern :core "floatp" (:lambda (t) (mu:eq :float (mu:type-of t))))
(mu:intern :core "structp" (:lambda (t) (mu:eq :struct (mu:type-of t))))

;;;
;;; syntactic sugar
;;;
(mu:intern :core "%orf"
  (:lambda (arg arg1)
    (:if arg arg arg1)))

(mu:intern :core "%andf"
  (:lambda (arg arg1)
    (:if arg arg1 ())))

;;;
;;; apply
;;;
(mu:intern :core "apply"
   (:lambda (fn args)
     (:if (core:functionp fn)
          (:if (core:core-function-p fn)
               (core:%function-apply fn args)
               (mu:apply fn args))
          (core:raise fn 'core:apply "not a function designator"))))

;;;
;;; apply closure to list of args => obj
;;;
(mu:intern :core "%function-apply"
   (:lambda (fn args)
      ((:lambda (env)
        (core:%mapc mu:fr-push env)
        ((:lambda (value)
           (core:%mapc (:lambda (frame) (mu:fr-pop (mu:car frame))) env)
           value)
         ((:lambda (mu-fn)
             (mu:apply mu-fn (mu:eval (core:%lambda-arg-list fn args))))
          (core:%function-prop :fn fn))))
        (core:%function-prop :env fn))))

(mu:intern :core "%function-apply-quoted"
   (:lambda (fn args)
      ((:lambda (env)
        (core:%mapc mu:fr-push env)
        ((:lambda (value)
           (core:%mapc (:lambda (frame) (mu:fr-pop (mu:car frame))) env)
           value)
         ((:lambda (mu-fn)
             (mu:apply mu-fn args))
          (core:%function-prop :fn fn))))
        (core:%function-prop :env fn))))
