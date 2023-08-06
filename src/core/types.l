;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; type system
;;;
(mu:intern core :extern "ctypep"
   (:lambda (object)
      (:if (core:structp object)
           (mu:eq :ctype (mu:st-type object))
           ())))

(mu:intern core :extern "type-of"
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
;;; where the property-alist is (:name . :predicate)
;;;
(mu:intern core :intern "defctype"
   (:lambda (name properties)
      ((:lambda (str)
          (core:raise-unless core:listp properties 'core::defctype "not a property list")
          ((:lambda (symbol)
              (core:raise-unless core:null symbol 'core::defctype "type symbol bound")
              (mu:intern core::ctypes :extern str (mu:make-st :ctype `(,str ,properties))))
           (mu:ns-find core::ctypes :extern str)))
       (core:string name))))

(mu:intern core :intern "ctype-predicate-map"
   `(,(mu:cons :char core:charp)
     ,(mu:cons :cons core:consp)
     ,(mu:cons :fixnum core:fixnump)
     ,(mu:cons :float core:floatp) 
     ,(mu:cons :func core:functionp)
     ,(mu:cons :listp core:listp)
     ,(mu:cons :ns core:namespacep)
     ,(mu:cons :string core:stringp)
     ,(mu:cons :struct core:structp)
     ,(mu:cons :symbol (:lambda (t) (mu:eq :symbol (mu:type-of t))))
     ,(mu:cons :symbolp core:symbolp)
     ,(mu:cons :t (:lambda (t) :t))
     ,(mu:cons :vector core:vectorp)))

(mu:intern core :intern "ctype-predicate"
   (:lambda (type object)
      ((:lambda (predicate)
          (core:raise-when core:null type 'core::ctype-predicate "predicate not bound")
          (mu:apply predicate `(,object)))
       (mu:cdr (core:assoc type core::ctype-predicate-map)))))

(mu:intern core :intern "make-ctype"
    (:lambda (name property-values)
       ((:lambda (str)
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
                                 (:if (core::ctype-predicate type value)
                                      property-def
                                      (core:raise property-value 'core::make-ctype `(,type ,value "property type botch"))))
                              (mu:cdr property-def)
                              (mu:cdr property-value))
                             ()))
                     (mu:sv-ref (mu:st-vec (mu:sy-val type-symbol)) 1))))
                property-values))
            (mu:ns-find core::ctypes :extern str))
           (mu:make-st :ctype `(,str ,property-values)))
       (core:string name))))

(mu:intern core :intern "ctype-ref"
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
  '((:lambda  . :struct)
    (:nreqs   . :fixnum)
    (:fn      . :func)
    (:env     . :listp)))

(mu:intern core :extern "closurep"
  (:lambda (fn)
     (mu:eq (core:type-of fn) 'closure)))

;;; lambda
(core::defctype "lambda"
   '((:syms . :listp)
     (:reqs . :listp)
     (:rest . :symbolp)
     (:env  . :listp)))

(mu:intern core :extern "lambdap"
  (:lambda (fn)
     (mu:eq (core:type-of fn) 'lambda)))

;;; sequence
(core::defctype "seq"
  '((:type   . :symbol)
    (:vector . :vector)
    (:cons   . :cons)))
