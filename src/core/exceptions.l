;;;  Spdx-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; exceptions
;;;
(mu:untern core "%debug-core-symbols%")

(mu:intern core "exceptionp"
   (:lambda (t)
      (:if (core:structp t)
           (mu:eq :except (mu:st-type t))
           ())))

;;;
;;; [*exception descriptor*] #s(:except cond obj source reason env)
;;;
;;; cond:    condition type symbol
;;; object:  exception raised on object
;;; source:  function designator, usually a symbol
;;; reason:  extended condition, usually a string
;;; env:     dynamic environment, list of frames
;;;
(mu:intern core "make-exception"
  (:lambda (cond object source reason env)
    (mu:make-st :except
      `(,cond ,object ,source ,reason ,env))))

(mu:intern core "%exception-property"
  (:lambda (key exception)
    (mu:sv-ref
     (mu:st-vec exception)
      (mu:cdr
       (core:%assoc
        key
        '((:cond    . 0)
          (:object  . 1)
          (:source  . 2)
          (:reason  . 3)
          (:env     . 4)))))))

(mu:intern core "%debug-map-symbol"
   (:lambda (obj)
      (:if (mu:boundp 'core:%debug-core-symbols%)
           ()
           (mu:intern core "%debug-core-symbols%" `(mu:ns-syms core)))
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

(mu:intern core "%fmt-mapped-symbol"
   (:lambda (obj)
      ((:lambda (symbol)
          (core:%orf (mu:nth 0 symbol) (mu:nth 1 symbol)))
      (core:%debug-map-symbol obj))))

(mu:intern core "exceptf"
   (:lambda (stream fmt printenv except)
     ((:lambda (cond object source reason env)
         (:if (core:null fmt)
              (core:format stream "~S ~S ~S ~S~%" `(,cond ,object ,source ,reason))
              (core:format stream fmt `(,cond ,object ,source ,reason)))
         (:if printenv
              (core:%prog2
               (core:format stream "env:~%" ())
               (core:%mapc
                (:lambda (frame)
                   (core:format stream "  ~A~T(~A ~A)~%"
                                `(,(core:%fmt-mapped-symbol (mu:car frame))
                                  ,(mu:car frame)
                                  ,(mu:cdr frame))))
                (mu:cdr env)))
             ()))
     (core:%exception-property :cond except)
     (core:%exception-property :object except)
     (core:%exception-property :source except)
     (core:%exception-property :reason except)
     (core:%exception-property :env except))))

(mu:intern core "with-exception"
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
                                      (core:dropl (mu:frames) 4))))))
      (:lambda () (core:apply thunk ())))))

;;;
;;; exception flavors
;;;
(mu:intern core "raise"
   (:lambda (object source reason)
      (mu:raise
       (core:make-exception :error object source reason (core:dropl (mu:frames) 2))
       :except)))

(mu:intern core "raise-if"
  (:lambda (test obj source reason)
    (:if test
         (core:raise obj source reason)
         obj)))

(mu:intern core "raise-when"
  (:lambda (fn obj source reason)
    (:if (mu:apply fn `(,obj))
         (core:raise obj source reason)
         obj)))

(mu:intern core "raise-unless"
  (:lambda (fn obj source reason)
    (:if (mu:apply fn `(,obj))
         obj
         (core:raise obj source reason))))

(mu:intern core "warn"
  (:lambda (obj msg)
     (core:format mu:err-out ";;; warning: ~S ~A~%" `(,obj ,msg))
    obj))

(mu:intern core "warn-when"
  (:lambda (fn obj source reason)
    (:if (mu:apply fn `(,obj))
         (core:warn obj reason)
         obj)))

(mu:intern core "warn-unless"
  (:lambda (fn obj source reason)
    (:if (mu:apply fn `(,obj))
         obj
         (core:warn obj reason))))
