;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; types
;;;
(mu:intern :core "%core-types" (mu:make-ns :%ctypes))

(mu:intern :core "core-type-p"
   (:lambda (object)
      (:if (core:structp object)
           (mu:eq :<ctype> (mu:st-type object))
           ())))

(mu:intern :core "type-of"
   (:lambda (object)
      ((:lambda (type)
          (:if (mu:eq type :struct)
               (:if (mu:eq :<ctype> (mu:st-type object))
                    (mu:untern () (mu:sv-ref (mu:st-vec object) 0))
                    (mu:st-type object))
               (:if (core:stringp object)
                    'string
                    (mu:untern () (mu:sy-name type)))))
       (mu:type-of object))))

;;;
;;; ctype struct: #s(:<ctype> #(:t name property-alist))
;;;
;;; where the property-alist is ((:predicate . :predicate) ...)
;;;
(mu:intern :core "define-core-type"
   (:lambda (name properties)
      ((:lambda (str)
          (:if (core:listp properties)
               ((:lambda (symbol)
                   (:if (core:null symbol)
                        (mu:intern core:%core-types str (mu:make-st :<ctype> `(,str ,properties)))
                        (core:raise symbol 'core:define-core-type "type symbol bound")))
                (mu:ns-find core:%core-types str))
               (core:raise properties 'core:define-core-type "not a property list")))
       (core:string name))))

(mu:intern :core "%core-type-predicate-map%"
   `(,(mu:cons :char core:charp)
     ,(mu:cons :cons core:consp)
     ,(mu:cons :fixnum core:fixnump)
     ,(mu:cons :float core:floatp)
     ,(mu:cons :func core:functionp)
     ,(mu:cons :listp core:listp)
     ,(mu:cons :keyword core:keywordp)
     ,(mu:cons :string core:stringp)
     ,(mu:cons :struct core:structp)
     ,(mu:cons :symbol (:lambda (t) (mu:eq :symbol (mu:type-of t))))
     ,(mu:cons :symbolp core:symbolp)
     ,(mu:cons :t (:lambda (t) :t))
     ,(mu:cons :vector core:vectorp)))

(mu:intern :core "%core-type-predicate"
   (:lambda (type object)
      ((:lambda (predicate)
          (mu:apply predicate `(,object)))
       (mu:cdr (core:%assoc type core:%core-type-predicate-map%)))))

(mu:intern :core "%make-core-type"
    (:lambda (name property-values)
       ((:lambda (str)
           ((:lambda (type-symbol)
               (core:%mapc
                (:lambda (property-value)
                   (core:findl
                    (:lambda (property-def)
                       (:if (mu:eq (mu:car property-value) (mu:car property-def))
                            ((:lambda (type value)
                                (:if (core:%core-type-predicate type value)
                                     property-def
                                     (core:raise
                                      property-value
                                      'core:%make-core-type
                                      `(,type ,value "property type botch"))))
                             (mu:cdr property-def)
                             (mu:cdr property-value))
                            ()))
                    (mu:sv-ref (mu:st-vec (mu:sy-val type-symbol)) 1)))
                property-values))
            (mu:ns-find core:%core-types str))
           (mu:make-st :<ctype> `(,str ,property-values)))
        (core:string name))))

;;;
;;; returns () if not found
;;;
(mu:intern :core "%core-type-ref"
   (:lambda (property-name struct)
      (:if (core:core-type-p struct)
           (:if (core:keywordp property-name)
                (core:%assoc property-name (mu:sv-ref (mu:st-vec struct) 1))
                (core:raise struct 'core:%core-type-ref "not a core-type"))
           (core:raise property-name 'core:%core-type-ref "not a property name"))))
