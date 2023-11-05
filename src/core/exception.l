;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; exceptions
;;;
(mu:untern :core "%debug-core-symbols%")

;;;
;;; [*exception descriptor*] #s(:except cond value source reason env)
;;;
;;; cond:    condition type symbol
;;; value:  exception raised on value
;;; source:  function designator, usually a symbol
;;; reason:  extended condition, usually a string
;;; env:     dynamic environment, list of frames
;;;
(mu:intern :core "exceptp"
   (:lambda (ex)
      (mu:eq (core:type-of ex) 'except)))

(mu:intern :core "make-except"
   (:lambda (cond value source reason env)
      (core:%make-core-type "except"
        (core:%pair-list
         `(:cond   ,cond
           :value ,value
           :source ,source
           :reason ,reason
           :env    ,env)))))

(mu:intern :core "%except-prop"
   (:lambda (prop except)
      (:if (core:findl (:lambda (lambda-prop) (mu:eq prop lambda-prop)) '(:cond :value :source :reason :env))
           (core:%core-type-ref prop except)
           (core:raise prop 'core:%except-prop "not an except property"))))

(mu:intern :core "exceptf"
   (:lambda (stream format-string printenv except)
     ((:lambda (cond value source reason env)
         (:if (core:null format-string)
              (core:format stream "~S ~S ~S ~S~%" `(,cond ,value ,source ,reason))
              (core:format stream format-string `(,cond ,value ,source ,reason)))
         (:if (core:%andf printenv env)
              ((:lambda ()
                  (core:format stream "env:~%" ())
                  (core:%mapc
                   (:lambda (frame)
                      (core:format stream "  ~A~T(~A ~A)~%"
                                   `(,(core:%format-mapped-symbol (mu:car frame))
                                     ,(mu:car frame)
                                     ,(mu:cdr frame))))
                   (mu:cdr env))))
              ()))
      (core:%except-prop :cond except)
      (core:%except-prop :value except)
      (core:%except-prop :source except)
      (core:%except-prop :reason except)
      (core:%except-prop :env except))))

(mu:intern :core "with-exception"
  (:lambda (handler thunk)
     (mu:with-ex
      (:lambda (value condition source)
         (:if (mu:eq condition :except)
              (core:apply handler `(,value))
              (core:apply handler `(,(core:make-except
                                      condition
                                      value
                                      source
                                      "mu:raise"
                                      (mu:frames))))))
      (:lambda () (core:apply thunk ())))))

;;;
;;; exception flavors
;;;
(mu:intern :core "error"
   (:lambda (format-string value)
      (core:format mu:err-out format-string `(,value))
      (mu:raise
       (core:make-except :error value :error "" (mu:frames))
       :except)))

(mu:intern :core "raise"
   (:lambda (value source reason)
      (mu:raise
       (core:make-except :error value source reason (mu:frames))
       :except)))

(mu:intern :core "warn"
  (:lambda (value message)
     (core:format mu:err-out ";;; warning: ~S ~A~%" `(,value ,message))
    value))
