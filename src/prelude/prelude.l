;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; namespace
;;;
(mu:make-ns :prelude)

;;;
;;; types
;;;
(mu:intern :prelude "%init-ns"
   (:lambda ()
      ;;; function
      (prelude:define-type "funct"
          '((:lambda  . :struct)
            (:arity   . :fixnum)
            (:fn      . :func)
            (:env     . :listp)))

      ;;; lambda
      (prelude:define-type "lambda"
          '((:bound  . :listp)
            (:rest   . :symbolp)
            (:lexenv . :listp)))

      ;;; exception
      (prelude:define-type "except"
          '((:cond   . :symbolp)
            (:value . :t)
            (:source . :symbolp)
            (:reason . :string)
            (:env    . :listp)))

      ;;; namespace
      (prelude:define-type "namespc"
          '((:name    . :string)
            (:intern  . :keyword)
            (:extern  . :keyword)
            (:ch-mac  . :keyword)
            (:sy-mac  . :keyword)
            (:uses    . :listp)))

      ;;; vector
      (prelude:define-type "vector"
          '((:base    . :vector)
            (:disp    . :listp)
            (:bitvec  . :listp)))

      (mu:intern :prelude "%char-macro-ns%" (prelude:%gen-anon-ns))
      (mu:intern :prelude "%macro-ns%" (prelude:%gen-anon-ns))
      (mu:intern :prelude "%symbol-macro-ns%" (prelude:%gen-anon-ns))))

;;;
;;; predicates
;;;
(mu:intern :prelude "not" (:lambda (value) (prelude:null value)))
(mu:intern :prelude "atom" (:lambda (value) (prelude:not (prelude:consp value))))

(mu:intern :prelude "constantp"
   (:lambda (value)
      (:if (prelude:%orf (prelude:null value) (prelude:keywordp value))
           :t
           (:if (prelude:consp value)
                (mu:eq :quote (mu:car value))
                (prelude:not (prelude:symbolp value))))))

(mu:intern :prelude "numberp"
  (:lambda (n)
     (:if (prelude:fixnump n)
          :t
          (prelude:floatp n))))

(mu:intern :prelude "zerop"
   (:lambda (n)
     (:if (prelude:numberp n)
          (:if (mu:eq 0 n)
               :t
               (mu:eq 0.0 n))
          (prelude:raise n 'prelude:zerop "not a number"))))

(mu:intern :prelude "minusp"
   (:lambda (n)
      (:if (prelude:numberp n)
           (:if (prelude:floatp n)
                (mu:fl-lt n 0.0)
                (mu:fx-lt n 0))
           (prelude:raise n 'prelude:minusp "not a number"))))

(mu:intern :prelude "plusp"
   (:lambda (n)
      (:if (prelude:numberp n)
           (:if (prelude:floatp n)
                (mu:fl-lt 0.0 n)
                (mu:fx-lt 0 n))
           (prelude:raise n 'prelude:plusp "not a number"))))

;;;
;;; system classes
;;;
(mu:intern :prelude "charp" (:lambda (t) (mu:eq :char (mu:type-of t))))
(mu:intern :prelude "fixnump" (:lambda (t) (mu:eq :fixnum (mu:type-of t))))
(mu:intern :prelude "floatp" (:lambda (t) (mu:eq :float (mu:type-of t))))
(mu:intern :prelude "structp" (:lambda (t) (mu:eq :struct (mu:type-of t))))

;;;
;;; syntactic sugar
;;;
(mu:intern :prelude "%orf"
   (:lambda (arg arg1)
     (:if arg arg arg1)))

(mu:intern :prelude "%andf"
   (:lambda (arg arg1)
     (:if arg arg1 ())))

(mu:intern :prelude "%list-2"
   (:lambda (value1 value2)
      (mu:cons value1
         (mu:cons value2 ()))))        

;;;
;;; apply
;;;
(mu:intern :prelude "apply"
   (:lambda (function arg-list)
     (:if (prelude:functionp function)
          (:if (prelude:%prelude-function-p function)
               (prelude:%funct-apply function arg-list)
               (mu:apply function arg-list))
          (prelude:raise function 'prelude:apply "not a function designator"))))

;;;
;;; namespace components
;;;
(mu:intern :prelude "%gen-anon-ns"
   (:lambda ()
      (mu:make-ns (prelude:genkeyword #\%))))
