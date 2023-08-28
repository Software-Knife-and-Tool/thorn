;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core namespace
;;;
(mu:intern () "mu" (mu:map-ns "mu"))
(mu:intern () "core" (mu:make-ns "core" (mu:map-ns "mu")))

(mu:intern core "%core-char-macro-ns%" (mu:make-ns "%char-macro-ns%" ()))
(mu:intern core "%core-symbol-macro-ns%" (mu:make-ns "%symbol-macro-ns%" ()))

;;;
;;; core types
;;;
(mu:intern core "%init-core-ns"
   (:lambda ()
      ;;; keyword                 
      (core:define-core-type "keyword"
        '((:name . :vector)))

      ;;; closure
      (core:define-core-type "closure"
        '((:lambda  . :struct)
          (:nargs   . :fixnum)
          (:fn      . :func)
          (:env     . :listp)))

      ;;; lambda
      (core:define-core-type "lambda"
        '((:bound  . :listp)
          (:rest   . :symbolp)
          (:lexenv . :listp)))

      ;;; sequence
      (core:define-core-type "seq"
        '((:type   . :symbol)
          (:vector . :vector)
          (:cons   . :cons)))))

;;;
;;; predicates
;;;
(mu:intern core "not" (:lambda (t) (core:null t)))
(mu:intern core "atom"
  (:lambda (obj)
    (core:not (core:consp obj))))

(mu:intern core "numberp"
  (:lambda (n)
     (:if (core:fixnump n)
          :t
          (core:floatp n))))

(mu:intern core "zerop"
   (:lambda (n)
     (:if (core:numberp n)
          (:if (mu:eq 0 n)
               :t
               (mu:eq 0.0 n))
          (core:raise n 'core:zerop "not a number"))))

(mu:intern core "minusp"
   (:lambda (n)
      (:if (core:numberp n)
           (:if (core:floatp n)
                (mu:fl-lt n 0.0)
                (mu:fx-lt n 0))
           (core:raise n 'core:minusp "not a number"))))
   
(mu:intern core "plusp"
   (:lambda (n)
      (:if (core:numberp n)
           (:if (core:floatp n)
                (mu:fl-lt 0.0 n)
                (mu:fx-lt 0 n))
           (core:raise n 'core:plusp "not a number"))))

;;;
;;; system classes
;;;
(mu:intern core "charp" (:lambda (t) (mu:eq :char (mu:type-of t))))
(mu:intern core "fixnump" (:lambda (t) (mu:eq :fixnum (mu:type-of t))))
(mu:intern core "floatp" (:lambda (t) (mu:eq :float (mu:type-of t))))
(mu:intern core "functionp" (:lambda (t) (mu:eq :func (mu:type-of t))))
(mu:intern core "namespacep" (:lambda (t) (mu:eq :ns (mu:type-of t))))
(mu:intern core "structp" (:lambda (t) (mu:eq :struct (mu:type-of t))))
(mu:intern core "vectorp" (:lambda (t) (mu:eq :vector (mu:type-of t))))

;;;
;;; syntactic sugar
;;;
(mu:intern core "%orf"
  (:lambda (arg arg1)
    (:if arg arg arg1)))

(mu:intern core "%andf"
  (:lambda (arg arg1)
    (:if arg arg1 ())))

;;;
;;; core apply
;;;
(mu:intern core "%appliablep"
   (:lambda (fn)
      (core:%orf (core:closurep fn) (core:functionp fn))))

(mu:intern core "apply"
   (:lambda (fn args)
     (:if (core:%appliablep fn)
          (:if (core:closurep fn)
               (core:%closure-apply fn args)
               (mu:apply fn args))
          (core:raise fn 'core:apply "not a function designator"))))

;;;
;;; fixpoint loop
;;;
(mu:intern core "fix"
   (:lambda (fn list)
      (:if (core:%appliablep fn)
           (:if (core:listp list)
                ((:lambda (fn)
                    (mu:fix fn list))
                 (:if (core:closurep fn)
                      (:lambda (arg) (core:apply fn `(,arg)))
                      fn))
                (core:raise list 'core:fix "not a list"))
           (core:raise fn 'core:fix "not a function designator"))))
