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
                    (mu:untern () :extern (mu:sv-ref (mu:st-vec object) 0))
                    (mu:untern () :extern (mu:st-type object)))
               (mu:untern () :extern (mu:sy-name type))))
       (mu:type-of object))))

;;;
;;; ctype struct: #s(:ctype #(:t name property-alist))
;;;
;;; where the property-alist is (:name . :type)
;;;
(mu:intern core::ns :intern "defctype"
   (:lambda (name properties)
      (core:raise-unless core:stringp name 'core::defctype "not a string")
      (core:raise-unless core:listp properties 'core::defctype "not a property list")
      ((:lambda (symbol)
          (core:raise-unless core:null symbol 'core::defctype "type symbol bound")
          (mu:intern core::ctypes :extern name (mu:make-st :ctype `(,name ,properties))))
       (mu:ns-find core::ctypes :extern name))))

(mu:intern core::ns :intern "make-ctype"
   (:lambda (name property-values)
      (core:raise-unless core:stringp name 'core::make-ctype "not a type name")
      (core:raise-unless core:listp property-values 'core::make-ctype "not a property list")
      ((:lambda (type-symbol)
          (core:raise-when core:null type-symbol 'core::make-ctype "type unbound")
          (core:mapc
           (:lambda (property-value)
              ((:lambda (property-def)
                  (core:raise-when core:null property-def 'core::make-ctype "undefined property"))
               (core:findl
                (:lambda (property-def)
                   (:if (mu:eq (mu:car property-value) (mu:car property-def))
                        ((:lambda (type value)
                            (:if (core::or (mu:eq :t type) (mu:eq (mu:type-of value) type))
                                 property-def
                                 (:if (core::and (mu:eq :listp type) (core:listp value))
                                      property-def
                                      (:if (core::and (mu:eq :symbolp type) (core:symbolp value))
                                           property-def
                                           (core:raise property-value 'core::make-ctype "property type botch")))))
                         (mu:cdr property-def)
                         (mu:cdr property-value))
                        ()))
                (mu:sv-ref (mu:st-vec (mu:sy-val type-symbol)) 1))))
           property-values))
       (mu:ns-find core::ctypes :extern name))
      (mu:make-st :ctype `(,name ,property-values))))

(mu:intern core::ns :intern "ctype-ref"
   (:lambda (property-name struct)
      (core:raise-unless core:ctypep struct 'core::ctype-ref "not a ctype")
      (core:raise-unless core:keywordp property-name 'core::ctype-ref "not a property name")
      ((:lambda (property)
          (core:raise-when core:null property-name 'core::ctype-ref "property not bound")
          (mu:cdr property))
       (core:assoc property-name (mu:sv-ref (mu:st-vec struct) 1)))))

;;;
;;; core ctypes
;;;

;;; long keyword
(core::defctype "keyword"
  '((:name . :vector)))

;;; closure
(core::defctype "closure"
  '((:lambda . :struct)
    (:nreqs  . :fixnum)
    (:fn     . :func)))

;;; lambda
(core::defctype "lambda"
   '((:syms   . :listp)
     (:reqs   . :listp)
     (:rest   . :symbolp)
     (:env    . :listp)))

(mu:intern core::ns :extern "closurep"
  (:lambda (fn)
     (mu:eq (core:type-of fn) 'closure)))

;;; sequence
(core::defctype "seq"
  '((:type   . :symbol)
    (:vector . :vector)
    (:cons   . :cons)))
