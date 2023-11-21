;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core vectors
;;;
(mu:intern :core "vectorp"
   (:lambda (vector)
     (:if (mu:eq :vector (mu:type-of vector))
          :t
          (mu:eq 'vector (core:type-of vector)))))

(mu:intern :core "make-vector"
   (:lambda (list)
     (:if (core:listp list)
          (mu:make-sv (core:%specialized-vector-type list) list)
          (core:raise list 'core:make-vector "not a list"))))

(mu:intern :core "slice"
   (:lambda (vector start length)
     (:if (core:typep vector :vector)
          (:if (core:fixnump start)
               (:if (core:%orf (core:minusp start) (mu:fx-lt (core:vector-length vector) start))
                    (core:raise start 'core:slice "illegal start")
                    (:if (core:fixnump length)
                         (:if (core:%orf (core:minusp length) (mu:fx-lt (core:vector-length vector) length))
                              (core:raise length 'core:slice "illegal length")
                              (core:%make-vector vector start length))
                         (core:raise start 'core:slice "not a fixnum")))
               (core:raise length 'core:slice "not a fixnum"))
          (core:raise vector 'core:slice "not a base vector"))))

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
               (:if (core:%orf (core:minusp nth) (mu:fx-lt (core:vector-length vector) nth))
                    (core:raise length 'core:vector-ref "illegal index")
                    (:if (mu:eq :vector (mu:type-of vector))
                         (mu:sv-ref vector nth)
                         ((:lambda (base start)
                            (mu:sv-ref base (mu:fx-add start nth)))
                          (core:%vector-prop :base vector)
                          (core:%vector-prop :start vector))))
               (core:raise nth 'core:vector-ref "not a fixnum"))
          (core:raise prop 'core:vector-ref "not a vector"))))

;;;
;;; write
;;;
(mu:intern :core "%vector-write"
   (:lambda (vector escape stream)
      (:if (mu:eq :vector (mu:type-of vector))
           (mu:write vector escape stream)
           ((:lambda (length)
               (core:format stream "#(" ())
               (mu:fix
                (:lambda (index)
                   (:if (mu:fx-lt index length)
                        ((:lambda ()
                            (core:write (core:vector-ref vector index) escape stream)
                            (:if (mu:fx-lt index (core:1- length))
                                 (core:format stream " " ())
                                 ())
                            (core:1+ index)))
                        index))
                0)
               (core:format stream ")" ()))
            (core:%vector-prop :length vector)))
      ()))

;;;
;;; implementation
;;;
(mu:intern :core "%make-vector"
   (:lambda (vector start length)
     (core:%make-core-type "vector"
          (core:%pair-list
           `(:base   ,vector
             :start  ,start
             :length ,length)))))

(mu:intern :core "%specialized-vector-type"
   (:lambda (list)
     ((:lambda (type)
        (:if (core:findl (:lambda (stype) (mu:eq type stype)) '(:t :char :fixnum :float))
             (:if (mu:eq type :fixnum)
                  ((:lambda (is-byte)
                     (:if is-byte
                          :byte
                          :fixnum))
                   (core:foldl
                    (:lambda (elt acc)
                      (:if (core:%andf (core:not (core:minusp elt)) (mu:fx-lt elt 256))
                           acc
                           ()))
                      :t
                      list))
                  type)
             :t))
      (core:foldl
       (:lambda (elt acc)
          (:if (mu:eq :t acc)
               :t
               (:if (mu:eq acc (mu:type-of elt))
                    acc
                    ())))
      (mu:type-of (mu:car list))
      (mu:cdr list)))))

(mu:intern :core "%vector-prop"
   (:lambda (prop vector)
      (:if (core:findl (:lambda (vec-prop) (mu:eq prop vec-prop)) '(:base :start :length))
           (mu:cdr (core:%core-type-ref prop vector))
           (core:raise prop 'core:%vector-prop "not a vector property"))))

(mu:intern :core "%map-vector"
  (:lambda (fn vector)
    (mu:fix
     (:lambda (nth)
       (:if nth
            (:if (mu:eq nth (mu:sv-len vector))
                 ()
                 ((:lambda ()
                     (core:apply fn (mu:cons (mu:sv-ref vector nth) ()))
                     (mu:fx-add 1 nth))))
            ()))
     0)))
