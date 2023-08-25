;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; fixnums
;;;
(mu:intern core "1+"
   (:lambda (n)
     (core:raise-unless core:fixnump n 'core:1+ "not a fixnum")
     (mu:fx-add n 1)))

(mu:intern core "1-"
   (:lambda (n)
     (core:raise-unless core:fixnump n 'core:1- "not a fixnum")
     (mu:fx-sub n 1)))

(mu:intern core "truncate"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:truncate "not a fixnum")
     (core:raise-unless core:fixnump m 'core:truncate "not a fixnum")
     `(,(mu:fx-div n m) ,(mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))))

;;; round to negative infinity
(mu:intern core "floor"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:floor "not a fixnum")
     (core:raise-unless core:fixnump m 'core:rem "not a fixnum")
     (mu:cons (mu:fx-div n m) (mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))))

;;; round to positive infinity
(mu:intern core "ceiling"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:ceiling "not a fixnum")
     (core:raise-unless core:fixnump m 'core:ceiling "not a fixnum")
     (mu:cons (mu:fx-div n m) (mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))))

(mu:intern core "mod"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:mod "not a fixnum")
     (core:raise-unless core:fixnump m 'core:mod "not a fixnum")
     (mu:cdr (core:floor m n))))

(mu:intern core "rem"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:rem "not a fixnum")
     (core:raise-unless core:fixnump m 'core:rem "not a fixnum")
     (mu:cd (core:truncate m n))))

;;; round to nearest
(mu:intern core "round"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:round "not a fixnum")
     (core:raise-unless core:fixnump m 'core:round "not a fixnum")
     (mu:fx-sub (mu:fx-div n m) (mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))))

(mu:intern core "ash"
   (:lambda (n count)
     (core:raise-unless core:fixnump n 'core:ash "not a fixnum")
     (core:raise-unless core:fixnump count 'core:ash "not a fixnum")
     (mu:car
      (mu:fix
       (:lambda (loop)
          ((:lambda (n count)
              (:if (core:zerop count)
                   loop
                   (:if (core:minusp count) 
                        (mu:cons (mu:fx-div n 2) (core:1+ count))
                        (mu:cons (mu:fx-mul n 2) (core:1- count)))))
           (mu:car loop)
           (mu:cdr loop)))
       (mu:cons n count)))))
