;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core namespace
;;;
(mu:make-ns "core" (mu:map-ns "mu"))
(mu:intern (mu:map-ns "core") :intern "ns" (mu:map-ns "core"))

;;;
;;; predicates
;;;
(mu:intern core::ns :extern "null" (:lambda (t) (mu:eq () t)))
(mu:intern core::ns :extern "not" (:lambda (t) (core:null t)))
(mu:intern core::ns :extern "charp" (:lambda (t) (mu:eq :char (mu:type-of t))))
(mu:intern core::ns :extern "consp" (:lambda (t) (mu:eq :cons (mu:type-of t))))
(mu:intern core::ns :extern "exceptionp"
           (:lambda (t)
             (:if (core:structp t)
                  (mu:eq :except (mu:st-type t))
                  ())))
(mu:intern core::ns :extern "fixnump" (:lambda (t) (mu:eq :fixnum (mu:type-of t))))
(mu:intern core::ns :extern "floatp" (:lambda (t) (mu:eq :float (mu:type-of t))))
(mu:intern core::ns :extern "functionp" (:lambda (t) (mu:eq :func (mu:type-of t))))
(mu:intern core::ns :extern "namespacep" (:lambda (t) (mu:eq :ns (mu:type-of t))))
(mu:intern core::ns :extern "streamp" (:lambda (t) (mu:eq :stream (mu:type-of t))))
(mu:intern core::ns :extern "structp" (:lambda (t) (mu:eq :struct (mu:type-of t))))
(mu:intern core::ns :extern "symbolp" (:lambda (t) (mu:eq :symbol (mu:type-of t))))
(mu:intern core::ns :extern "vectorp" (:lambda (t) (mu:eq :vector (mu:type-of t))))
(mu:intern core::ns :extern "listp" (:lambda (t) (:if (core:consp t) :t (core:null t))))
(mu:intern core::ns :extern "keywordp" (:lambda (t) (:if (mu:keyp t) :t (core:null t))))
(mu:intern core::ns :extern "sequencep" (:lambda (t) (:if (core:listp t) :t (core:vectorp t))))

(mu:intern core::ns :extern "atom"
  (:lambda (obj)
    (core:not (core:consp obj))))

(mu:intern core::ns :extern "dottedp"
  (:lambda (obj)
    (:if (core:consp obj)
         (core:atom (mu:cdr obj))
         ())))
                    
(mu:intern core::ns :extern "boundp" mu:boundp)
(mu:intern core::ns :extern "fboundp"
  (:lambda (symbol)
    (:if (core:boundp symbol)
         (core:functionp (mu:sy-val symbol))
         ())))

(mu:intern core::ns :extern "uninternedp"
  (:lambda (sym)
    (core:null (mu:sy-ns sym))))

(mu:intern core::ns :extern "closurep"
  (:lambda (fn)
    (:if (mu:eq :struct (mu:type-of fn))
         (mu:eq (mu:st-type fn) :closure)
         ())))

(mu:intern core::ns :extern "numberp"
  (:lambda (n)
    (:if (core:fixnump n) :t (core:floatp n))))

(mu:intern core::ns :extern "zerop"
  (:lambda (n)
    (:if (mu:eq 0 n)
         :t
         (mu:eq 0.0 n))))

;;;
;;; syntactic sugar
;;;
(mu:intern core::ns :intern "or"
  (:lambda (arg arg1)
    (:if arg arg arg1)))

(mu:intern core::ns :intern "and"
  (:lambda (arg arg1)
    (:if arg arg1 ())))

(mu:intern core::ns :intern "prog1"
  (:lambda (arg1 arg2)
    arg1))

(mu:intern core::ns :intern "prog2"
  (:lambda (arg1 arg2)
    arg2))

(mu:intern core::ns :intern "princ"
  (:lambda (obj)
    (mu:write obj () mu:std-out)))

(mu:intern core::ns :intern "print"
  (:lambda (obj)
    (mu:write obj :t mu:std-out)))

;;;
;;; core apply
;;;
(mu:intern core::ns :extern "apply"
  (:lambda (fn args)
    (core:raise-unless
      (:lambda (fn) (core::or (core:functionp fn) (core:closurep fn)))
      fn
      'core:apply "not a function or closure")
     (:if (core:functionp fn)
          (mu:apply fn args)
          (core::closure-apply fn args))))
