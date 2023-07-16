;;;  Spdx-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; exceptions
;;;
(mu:intern core::ns :extern "exceptionp"
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
(mu:intern core::ns :extern "make-exception"
  (:lambda (cond object source reason env)
    (mu:make-st :except
      `(,cond ,object ,source ,reason ,env))))

(mu:intern core::ns :intern "exception-property"
  (:lambda (key exception)
    (mu:sv-ref
     (mu:st-vec exception)
      (mu:cdr
       (core:assoc
        key
        '((:cond    . 0)
          (:object  . 1)
          (:source  . 2)
          (:reason  . 3)
          (:env     . 4)))))))

(mu:intern core::ns :extern "exceptionf"
   (:lambda (stream fmt printenv except)
     ((:lambda (cond object source reason env)
         (:if (core:null fmt)                
              (core:format stream "~S ~S ~S ~S~%" `(,cond ,object ,source ,reason))
              (core:format stream fmt `(,cond ,object ,source ,reason)))
         (:if printenv
              ((:lambda ()
                  (core:format stream "environment:~%" ())
                  (core:mapc
                   (:lambda (frame)
                      (core:format stream "(~A)~%" frame))
                   env)))
             ()))
     (core::exception-property :cond except)
     (core::exception-property :object except)
     (core::exception-property :source except)
     (core::exception-property :reason except)
     (core::exception-property :env except))))

(mu:intern core::ns :extern "with-exception"
  (:lambda (handler thunk)
    (mu:with-ex
       (:lambda (object condition)
          (:if (mu:eq condition :except)
               (core:apply handler (mu:cons source ()))
               (core:apply
                handler
                `(,(core:make-exception condition object 'core::repl "mu" (core:dropl (mu::frames) 4))))))
       (:lambda () (core:apply thunk ())))))

;;;
;;; exception flavors
;;;
(mu:intern core::ns :extern "raise"
   (:lambda (object source reason)
      (mu:raise
       (core:make-exception :error object source reason (core:dropl (mu::frames) 2))
       :except)))

(mu:intern core::ns :extern "raise-if"
  (:lambda (test obj source reason)
    (:if test
         (core:raise obj source reason)
         obj)))

(mu:intern core::ns :extern "raise-when"
  (:lambda (fn obj source reason)
    (:if (mu:apply fn `(,obj))
         (core:raise obj source reason)
         obj)))

(mu:intern core::ns :extern "raise-unless"
  (:lambda (fn obj source reason)
    (:if (mu:apply fn `(,obj))
         obj
         (core:raise obj source reason))))

;;;
;;; mostly for debugging
;;;
(mu:intern core::ns :intern "eprinc"
  (:lambda (obj)
    (mu:write obj () mu:err-out)))

(mu:intern core::ns :intern "eprint"
  (:lambda (obj)
    (mu:write obj :t mu:err-out)))

(mu:intern core::ns :extern "warn"
  (:lambda (obj msg)
     (core:format mu:err-out "warning: ~S ~A~%" `(,obj ,msg))
    obj))

(mu:intern core::ns :intern "repl"
  (:lambda ()
     (mu:fix
      (:lambda (loop)
        (:if (core:eof :t)
             loop
             (core:with-exception
              (:lambda (ex)
                 (core:exceptionf mu:err-out "exception: condition: ~A on ~A by ~S from ~A~%" () ex)
                 (core:null loop))
               (:lambda ()
                 (core:format :t "core> " ())
                 (mu:flush mu:std-out)
                 ((:lambda (form)
                    (:if (core:eof :t)
                         ()
                         ((:lambda (form)
                            (core:format :t "[~A] ~S~%" `(,(mu:type-of form) ,form))
                            (core:null loop))
                          (mu:eval (core:compile form)))))
                  (core:read :t () ()))))))
      ())))
