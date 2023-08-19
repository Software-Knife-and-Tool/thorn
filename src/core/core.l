;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core namespace
;;;
(mu:make-ns "core" (mu:map-ns "mu"))
(mu:intern () :extern "core" (mu:map-ns "core"))
(mu:intern core :intern "core-types" (mu:make-ns "core-types" core))
(mu:intern core :intern "core-char-macro-ns" (mu:make-ns "core:char-macro-ns" ()))
(mu:intern core :intern "core-symbol-macro-ns" (mu:make-ns "core:symbol-macro-ns" ()))

;;;
;;; predicates
;;;
(mu:intern core :extern "not" (:lambda (t) (core:null t)))
(mu:intern core :extern "atom"
  (:lambda (obj)
    (core:not (core:consp obj))))

(mu:intern core :extern "numberp"
  (:lambda (n)
     (:if (core:fixnump n)
          :t
          (core:floatp n))))

(mu:intern core :extern "zerop"
  (:lambda (n)
    (:if (mu:eq 0 n)
         :t
         (mu:eq 0.0 n))))

;;;
;;; system classes
;;;
(mu:intern core :extern "charp" (:lambda (t) (mu:eq :char (mu:type-of t))))
(mu:intern core :extern "fixnump" (:lambda (t) (mu:eq :fixnum (mu:type-of t))))
(mu:intern core :extern "floatp" (:lambda (t) (mu:eq :float (mu:type-of t))))
(mu:intern core :extern "functionp" (:lambda (t) (mu:eq :func (mu:type-of t))))
(mu:intern core :extern "namespacep" (:lambda (t) (mu:eq :ns (mu:type-of t))))
(mu:intern core :extern "structp" (:lambda (t) (mu:eq :struct (mu:type-of t))))
(mu:intern core :extern "vectorp" (:lambda (t) (mu:eq :vector (mu:type-of t))))

;;;
;;; core classes
;;;
(mu:intern core :extern "sequencep" (:lambda (t) (:if (core:listp t) :t (core:vectorp t))))

;;;
;;; syntactic sugar
;;;
(mu:intern core :intern "orf"
  (:lambda (arg arg1)
    (:if arg arg arg1)))

(mu:intern core :intern "andf"
  (:lambda (arg arg1)
    (:if arg arg1 ())))

(mu:intern core :intern "prog1"
  (:lambda (arg1 arg2)
    arg1))

(mu:intern core :intern "prog2"
  (:lambda (arg1 arg2)
    arg2))

;;;
;;; core apply
;;;
(mu:intern core :intern "appliablep"
   (:lambda (fn)
      (core::orf (core:closurep fn) (core:functionp fn))))

(mu:intern core :extern "apply"
   (:lambda (fn args)
     (core:raise-unless core::appliablep fn 'core:apply "not a function designator")
     (:if (core:closurep fn)
          (core::closure-apply fn args)
          (mu:apply fn args))))

;;;
;;; fixpoint loop
;;;
(mu:intern core :extern "fix"
   (:lambda (fn list)
     (core:raise-unless core::appliablep fn 'core:fix "not a function designator")
     (core:raise-unless core:listp list 'core:fix "not a list")
     (:if (core:closurep fn)
          (mu:fix (:lambda (arg) (core:apply fn `(,arg))) list)
          (mu:fix fn list))))
