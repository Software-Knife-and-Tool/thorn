;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; closures
;;;

;;;
;;; [*closure descriptor*] #s(:closure env nreqs rest func)
;;;
;;; env:   list of frame state structs
;;; nreqs: number of required arguments
;;; rest:  rest symbol or nil
;;; func:  function, mu applyable
;;;
(mu:intern core::ns :intern "closure-property"
  (:lambda (key closure)
    (core:raise-unless core:closurep closure 'core::closure-property "is not a closure")
    (mu:sv-ref
     (mu:st-vec closure)
     (mu:cdr
      (core:assoc
       key
       '((:env   . 0)
         (:nreqs . 1)
         (:rest  . 2)
         (:func  . 3)))))))

(mu:intern core::ns :intern "make-closure"
  (:lambda (env nreqs rest func)
    (mu:make-st :closure
      `(,env ,nreqs ,rest ,func))))

;;;
;;; create a closure
;;;
(mu:intern core::ns :intern "closure"
  (:lambda (fn)
    ((:lambda (lambda-desc)
       (core:raise-when core:null lambda-desc 'core::closure "closing bare function")
       ((:lambda (desc body)
          (mu:compile `(:lambda ,(core::ctype-ref :syms desc) ,desc ,@(mu:cdr (core::fn-form fn)))))
        (core::make-lambda
         (core::ctype-ref :syms lambda-desc)
         (core::ctype-ref :reqs lambda-desc)
         (core::ctype-ref :rest lambda-desc)
         (core::ctype-ref :env lambda-desc)
         (core:mapcar
          (:lambda (desc)
             (mu:fr-get (core::ctype-ref :frame desc)))
          (core::ctype-ref :env lambda-desc)))
        (mu:cdr (core::fn-form fn))))
     (core::fn-lambda-desc fn))))
