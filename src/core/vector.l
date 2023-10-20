;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core vectors
;;;
(mu:intern :core "vectorp"
   (:lambda (vec)
     ((:lambda (type)
        (:if (mu:eq type :vector)
          :t
          (:if (core:core-type-p vec)
               (mu:eq 'vector (core:type-of vec))
               ())))
      (mu:type-of vec))))

(mu:intern :core "make-vector"
   (:lambda (list)
     (:if (core:listp list)
          ((:lambda (vec)
              (core:%make-vector vec 0 (mu:sv-len vec)))
           (mu:make-sv :t list))
          (core:raise list 'core:make-vector "not a list"))))

(mu:intern :core "slice"
   (:lambda (vec start length)
     (:if (core:vectorp vec)
          (:if (core:fixnump start)
               (:if (core:fixnump length)
                    (core:%make-vector vec start length)
                    (core:raise start 'core:slice "not a fixnum"))
               (core:raise length 'core:slice "not a fixnum"))
          (core:raise vec 'core:slice "not a vector"))))

(mu:intern :core "vector-length"
   (:lambda (vector)
     (:if (core:vectorp vector)
          (:if (mu:eq :vector (mu:type-of vector))
               (mu:sv-len vector)
               (core:%vector-prop :length vector))
          (core:raise vector 'core:make-vector "not a vector"))))

(mu:intern :core "vector-ref"
   (:lambda (vector nth)
     (:if (core:vectorp vector)
          (:if (core:fixnump nth)
               (:if (mu:eq :vector (mu:type-of vector))
                    (mu:sv-ref vector nth)
                    ((:lambda (base start)
                       (core:vector-ref base (mu:fx-add start nth)))
                     (core:%vector-prop :base vector)
                     (core:%vector-prop :start vector)))
               (core:raise nth 'core:vector-ref "not a fixnum"))
          (core:raise prop 'core:vector-ref "not a vector"))))

;;;
;;; implementation
;;;
(mu:intern :core "%make-vector"
   (:lambda (vec start length)
     (core:%make-core-type "vector"
          (core:%pair-list
           `(:base   ,vec
             :start  ,start
             :length ,length)))))

(mu:intern :core "%vector-prop"
   (:lambda (prop vec)
      (:if (core:findl (:lambda (vec-prop) (mu:eq prop vec-prop)) '(:base :start :length))
           (mu:cdr (core:%core-type-ref prop vec))
           (core:raise prop 'core:%vector-prop "not a vector property"))))

(mu:intern :core "%map-vector"
  (:lambda (fn vector)
    (mu:fix
     (:lambda (nth)
       (:if nth
            (:if (mu:eq nth (mu:sv-len vector))
                 ()
                 ((:lambda ()
                     (core:apply fn `(,(mu:sv-ref vector nth)))
                     (mu:fx-add 1 nth))))
            ()))
     0)))
