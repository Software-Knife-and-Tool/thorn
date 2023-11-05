;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; types
;;;
(mu:intern :core "%core-types" (mu:make-ns :%ctypes))

(mu:intern :core "core-type-p"
   (:lambda (value)
      (:if (core:structp value)
           (mu:eq :<ctype> (mu:st-type value))
           ())))

(mu:intern :core "type-of"
   (:lambda (value)
      ((:lambda (type)
          (:if (mu:eq type :struct)
               (:if (mu:eq :<ctype> (mu:st-type value))
                    (mu:untern () (mu:sv-ref (mu:st-vec value) 0))
                    (mu:st-type value))
               (:if (core:stringp value)
                    'string
                    (mu:untern () (mu:sy-name type)))))
       (mu:type-of value))))

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
   `(,(mu:cons :char (:lambda (t) (core:charp t)))
      ,(mu:cons :cons (:lambda (t) (core:consp t)))
      ,(mu:cons :fixnum (:lambda (t) (core:fixnump t)))
      ,(mu:cons :float (:lambda (t) (core:floatp t)))
      ,(mu:cons :func (:lambda (t) (core:functionp t)))
      ,(mu:cons :listp (:lambda (t) (core:listp t)))
      ,(mu:cons :map (:lambda (t) (mu:eq :map (mu:type-of t))))
      ,(mu:cons :keyword (:lambda (t) (core:keywordp t)))
      ,(mu:cons :nmspace (:lambda (t) (core:namespacep t)))
      ,(mu:cons :string (:lambda (t) (core:stringp t)))
      ,(mu:cons :struct (:lambda (t) (core:structp t)))
      ,(mu:cons :symbol (:lambda (t) (mu:eq :symbol (mu:type-of t))))
      ,(mu:cons :symbolp (:lambda (t) (core:symbolp t)))
      ,(mu:cons :t (:lambda (t) :t))
      ,(mu:cons :vector (:lambda (t) (core:vectorp t)))))

(mu:intern :core "%core-type-predicate"
   (:lambda (type value)
      ((:lambda (predicate)
          (mu:apply predicate `(,value)))
       (mu:cdr (core:%assoc type core:%core-type-predicate-map%)))))

(mu:intern :core "%make-core-type"
    (:lambda (name property-values)
       ((:lambda (type-symbol)
           (core:%mapc
            (:lambda (property-value)
               (core:findl
                (:lambda (property-def)
                   (:if (mu:eq (mu:car property-value) (mu:car property-def))
                        ((:lambda (type value)
                            (:if (core:%core-type-predicate type value)
                                 property-def
                                 (core:raise property-value '%make-core-type "type mismatch")))
                         (mu:cdr property-def)
                         (mu:cdr property-value))
                        ()))
                (mu:sv-ref (mu:st-vec (mu:sy-val type-symbol)) 1)))
            property-values))
        (mu:ns-find core:%core-types name))
       (mu:make-st :<ctype> `(,name ,property-values))))

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
