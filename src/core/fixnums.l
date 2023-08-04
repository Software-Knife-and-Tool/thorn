;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; fixnums
;;;
(mu:intern core :extern "1+"
   (:lambda (n)
     (core:raise-unless core:fixnump n 'core:1+ "not a fixnum")
     (mu:fx-add n 1)))

(mu:intern core :extern "1-"
   (:lambda (n)
     (core:raise-unless core:fixnump n 'core:1- "not a fixnum")
     (mu:fx-sub n 1)))

(mu:intern core :extern "truncate"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:truncate "not a fixnum")
     (core:raise-unless core:fixnump m 'core:truncate "not a fixnum")
     `(,(mu:fx-div n m) ,(mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))))

;;; round to negative infinity
(mu:intern core :extern "floor"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:floor "not a fixnum")
     (core:raise-unless core:fixnump m 'core:rem "not a fixnum")
     (mu:cons (mu:fx-div n m) (mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))))

;;; round to positive infinity
(mu:intern core :extern "ceiling"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:ceiling "not a fixnum")
     (core:raise-unless core:fixnump m 'core:ceiling "not a fixnum")
     (mu:cons (mu:fx-div n m) (mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))))

(mu:intern core :extern "mod"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:mod "not a fixnum")
     (core:raise-unless core:fixnump m 'core:mod "not a fixnum")
     (mu:cdr (core:floor m n))))

(mu:intern core :extern "rem"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:rem "not a fixnum")
     (core:raise-unless core:fixnump m 'core:rem "not a fixnum")
     (mu:cd (core:truncate m n))))

;;; round to nearest
(mu:intern core :extern "round"
   (:lambda (n m)
     (core:raise-unless core:fixnump n 'core:round "not a fixnum")
     (core:raise-unless core:fixnump m 'core:round "not a fixnum")
     (mu:fx-sub (mu:fx-div n m) (mu:fx-sub n (mu:fx-mul m (mu:fx-div n m))))))
