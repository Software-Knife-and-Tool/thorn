;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; fixnums
;;;
(mu:intern :core "1+"
   (:lambda (n)
     (:if (core:fixnump n)
          (mu:fx-add n 1)
          (core:raise n 'core:1+ "not a fixnum"))))

(mu:intern :core "1-"
   (:lambda (n)
     (:if (core:fixnump n)
          (mu:fx-sub n 1)
          (core:raise n 'core:1- "not a fixnum"))))

(mu:intern :core "truncate"
   (:lambda (n m)
     (:if (core:fixnump n)
          (:if (core:fixnump m)
               (mu:cons (mu:fx-div n m) (mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))
               (core:raise m 'core:truncate "not a fixnum"))
          (core:raise n 'core:truncate "not a fixnum"))))

;;; round to negative infinity
(mu:intern :core "floor"
   (:lambda (n m)
     (:if (core:fixnump n)
          (:if (core:fixnump m)
               (mu:cons (mu:fx-div n m) (mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))
               (core:raise m 'core:rem "not a fixnum"))
          (core:raise n 'core:floor "not a fixnum"))))

;;; round to positive infinity
(mu:intern :core "ceiling"
   (:lambda (n m)
     (:if (core:fixnump n)
          (:if (core:fixnump m)
               (mu:cons (mu:fx-div n m) (mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))
               (core:raise m 'core:ceiling "not a fixnum"))
          (core:raise n 'core:ceiling "not a fixnum"))))

(mu:intern :core "mod"
   (:lambda (n m)
     (:if (core:fixnump n)
          (:if (core:fixnump m)
               (mu:cdr (core:floor m n))
               (core:raise m 'core:mod "not a fixnum"))
          (core:raise n 'core:mod "not a fixnum"))))

(mu:intern :core "rem"
   (:lambda (n m)
     (:if (core:fixnump n)
          (:if (core:fixnump m)
               (mu:cdr (core:truncate m n))
               (core:raise m 'core:rem "not a fixnum"))
          (core:raise n 'core:rem "not a fixnum"))))

;;; round to nearest
(mu:intern :core "round"
   (:lambda (n m)
     (:if (core:fixnump n)
          (:if (core:fixnump m)
               (mu:fx-sub (mu:fx-div n m) (mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))
               (core:raise m 'core:round "not a fixnum"))
          (core:raise n 'core:round "not a fixnum"))))

(mu:intern :core "ash"
   (:lambda (n count)
      (:if (core:fixnump n)
           (:if (core:fixnump count)
                (mu:car
                 (mu:fix
                  (:lambda (loop)
                     ((:lambda (n count)
                         (:if (core:zerop count)
                              loop
                              (:if (core:minusp count)
                                   (mu:cons (mu:fx-div n 2) (mu:fx-add 1 count))
                                   (mu:cons (mu:fx-mul n 2) (mu:fx-sub count 1)))))
                      (mu:car loop)
                      (mu:cdr loop)))
                  (mu:cons n count)))
                (core:raise count 'core:ash "not a fixnum"))
           (core:raise n 'core:ash "not a fixnum"))))
