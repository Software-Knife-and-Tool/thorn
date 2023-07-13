;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; type system
;;;
(mu:intern core::ns :extern "ctypep"
   (:lambda (object)
      (:if (core:structp object)
           (mu:eq :ctype (mu:st-type object))
           ())))

(mu:intern core::ns :extern "type-of"
   (:lambda (object)
      ((:lambda (type)
          (:if (mu:eq type :struct)
               (:if (mu:eq :ctype (mu:st-type object))
                    (mu:sv-ref (mu:st-vec object) 0)
                    type)
               type))
       (mu:type-of object))))

;;;
;;; ctype struct: #s(:ctype #(:t name slot-alist))
;;;
;;; where the slot-alist is (:name . :type)
;;;
(mu:intern core::ns :intern "def-ctype"
   (:lambda (name slots)
      (core:raise-unless core:stringp name 'core:def-ctype "not a string")
      (core:raise-unless core:listp slots 'core:def-ctype "not a slot list")
      ((:lambda (symbol)
          (core:raise-unless core:null symbol 'core:def-ctype "type symbol bound")
          (mu:intern core::ctypes :extern name (mu:make-st :ctype `(,name ,slots))))
       (mu:ns-find core::ctypes :extern name))))

(mu:intern core::ns :intern "make-ctype"
   (:lambda (name slot-defs)
      (core:raise-unless core:stringp name 'core:make-ctype "not a type name")
      (core:raise-unless core:listp slot-defs 'core:make-ctype "not a slot list")
      ((:lambda (symbol)
          (core:raise-when core:null symbol 'core:make-ctype "type unbound")
          (mu:make-st :ctype `(,name ,slot-defs)))
       (mu:ns-find core::ctypes :extern name))))

(mu:intern core::ns :intern "slot-value"
   (:lambda (slot-name struct)
      (core:raise-unless core:ctypep struct 'core::slot-value "not a ctype")
      (core:raise-unless core:keywordp slot-name 'core:slot-value "not a slot name")
      ((:lambda (slot)
          (core:raise-if core:null slot 'core::slot-value "slot not bound")
          (mu:car slot))
       (core:assoc slot-name (mu:sv-ref (mu:st-vec struct) 1)))))

;;;
;;; core ctypes
;;;

;;; exception
(core::def-ctype "core:ex"
  '((:cond . :symbol)
    (:object  . :t)
    (:source  . :symbol)
    (:reason  . :vector)
    (:env     . :cons)))

;;; string
(core::def-ctype "core:st"
  '((:text . :vector)))

;;; closure
(core::def-ctype "core:cl"
  '((:env   . :cons)
    (:nreqs . :fixnum)
    (:rest  . :symbol)
    (:func  . :func)
    (:apply . :func)))

;;; sequence
(core::def-ctype "core:sq"
  '((:type   . :symbol)
    (:vector . :vector)
    (:cons   . :cons)
    (:elt    . :func)))
