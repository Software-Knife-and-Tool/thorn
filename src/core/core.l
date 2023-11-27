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
            (:arity   . :fixnum)
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
            (:value . :t)
            (:source . :symbolp)
            (:reason . :string)
            (:env    . :listp)))

      ;;; core namespace
      (core:define-core-type "namespc"
          '((:name    . :string)
            (:intern  . :keyword)
            (:extern  . :keyword)
            (:ch-mac  . :keyword)
            (:sy-mac  . :keyword)
            (:uses    . :listp)))

      ;;; core vector
      (core:define-core-type "vector"
          '((:base    . :vector)
            (:disp    . :listp)
            (:bitvec  . :listp)))

      (mu:intern :core "%packages%" (core:%gen-anon-ns))
      (mu:intern :core "%char-macro-ns%" (core:%gen-anon-ns))
      (mu:intern :core "%macro-ns%" (core:%gen-anon-ns))
      (mu:intern :core "%symbol-macro-ns%" (core:%gen-anon-ns))))

;;;
;;; predicates
;;;
(mu:intern :core "not" (:lambda (value) (core:null value)))
(mu:intern :core "atom" (:lambda (value) (core:not (core:consp value))))

(mu:intern :core "constantp"
   (:lambda (value)
      (:if (core:%orf (core:null value) (core:keywordp value))
           :t
           (:if (core:consp value)
                (mu:eq :quote (mu:car value))
                (core:not (core:symbolp value))))))

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

(mu:intern :core "%list-2"
   (:lambda (value1 value2)
      (mu:cons value1
         (mu:cons value2 ()))))        

;;;
;;; apply
;;;
(mu:intern :core "apply"
   (:lambda (function arg-list)
     (:if (core:functionp function)
          (:if (core:core-function-p function)
               (core:%funct-apply function arg-list)
               (mu:apply function arg-list))
          (core:raise function 'core:apply "not a function designator"))))

;;;
;;; namespace components
;;;
(mu:intern :core "%gen-anon-ns"
   (:lambda ()
      (mu:make-ns (core:genkeyword #\%))))
