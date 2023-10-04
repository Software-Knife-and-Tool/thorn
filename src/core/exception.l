;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; exceptions
;;;
(mu:untern :core "%debug-core-symbols%")

;;;
;;; [*exception descriptor*] #s(:except cond obj source reason env)
;;;
;;; cond:    condition type symbol
;;; object:  exception raised on object
;;; source:  function designator, usually a symbol
;;; reason:  extended condition, usually a string
;;; env:     dynamic environment, list of frames
;;;
(mu:intern :core "exceptionp"
   (:lambda (ex)
      (mu:eq (core:type-of ex) 'except)))

(mu:intern :core "make-exception"
   (:lambda (cond object source reason env)
      (core:%make-core-type "except"
        (core:%pair-list
         `(:cond   ,cond
           :object ,object
           :source ,source
           :reason ,reason
           :env    ,env)))))
 
(mu:intern :core "%exception-prop"
   (:lambda (prop except)
      (:if (core:findl (:lambda (lambda-prop) (mu:eq prop lambda-prop)) '(:cond :object :source :reason :env))
           (core:%core-type-ref prop except)
           (core:raise prop 'except-prop "not an except property"))))

(mu:intern :core "%debug-map-symbol"
   (:lambda (obj)
      (:if (mu:boundp 'core:%debug-core-symbols%)
           ()
           (mu:intern :core "%debug-core-symbols%" (mu:ns-syms :core)))
      (mu:fix
       (:lambda (list)
          (:if (core:consp list)
               ((:lambda (symbol)
                   (:if (mu:eq (mu:sy-val symbol) obj)
                        symbol
                        (mu:cdr list)))
                (mu:car list))
               list))
       core:%debug-core-symbols%)))

(mu:intern :core "%fmt-mapped-symbol"
   (:lambda (obj)
      ((:lambda (symbol)
          (core:%orf (mu:nth 0 symbol) (mu:nth 1 symbol)))
       (core:%debug-map-symbol obj))))

(mu:intern :core "exceptf"
   (:lambda (stream fmt printenv except)
     ((:lambda (cond object source reason env)
         (:if (core:null fmt)
              (core:format stream "~S ~S ~S ~S~%" `(,cond ,object ,source ,reason))
              (core:format stream fmt `(,cond ,object ,source ,reason)))
         (:if (core:%andf printenv env)
              ((:lambda ()
                  (core:format stream "env:~%" ())
                  (core:%mapc
                   (:lambda (frame)
                      (core:format stream "  ~A~T(~A ~A)~%"
                                   `(,(core:%fmt-mapped-symbol (mu:car frame))
                                     ,(mu:car frame)
                                     ,(mu:cdr frame))))
                   (mu:cdr env))))
              ()))
      (core:%exception-prop :cond except)
      (core:%exception-prop :object except)
      (core:%exception-prop :source except)
      (core:%exception-prop :reason except)
      (core:%exception-prop :env except))))

(mu:intern :core "with-exception"
  (:lambda (handler thunk)
     (mu:with-ex
      (:lambda (object condition source)
         (:if (mu:eq condition :except)
              (core:apply handler `(,object))
              (core:apply handler `(,(core:make-exception
                                      condition
                                      object
                                      source
                                      "mu:raise"
                                      (core:dropl (mu:frames) 0))))))
      (:lambda () (core:apply thunk ())))))

;;;
;;; exception flavors
;;;
(mu:intern :core "error"
   (:lambda (fmt object)
      (core:format mu:err-out fmt `(,object))                
      (mu:raise
       (core:make-exception :error object :error "" (core:dropl (mu:frames) 0))
       :except)))

(mu:intern :core "raise"
   (:lambda (object source reason)
      (mu:raise
       (core:make-exception :error object source reason (core:dropl (mu:frames) 0))
       :except)))

(mu:intern :core "warn"
  (:lambda (obj msg)
     (core:format mu:err-out ";;; warning: ~S ~A~%" `(,obj ,msg))
    obj))
