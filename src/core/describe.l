;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; describe
;;;
(mu:intern :core "%describe-function"
   (:lambda (fn stream)
      ((:lambda (view)
          (core:format stream
            "function: (~A bytes) :func ~A~%    name:  [~A]~%    arity: ~A~%    body:  ~A~%"
            `(,(mu:hp-size fn)
              ,(core:type-of fn)
              ,(mu:sv-ref view 2)
              ,(mu:sv-ref view 0)
              ,(mu:sv-ref view 1))))
       (mu:view fn))))

(mu:intern :core "%describe-core-type"
   (:lambda (core-type stream)
      (core:format stream
       "core-type: (~A bytes) :<ctype> ~A~%    type:  ~A~%    props: ~A~%"
       `(,(mu:hp-size core-type)
         ,(core:type-of core-type)
         ,(mu:sv-ref (mu:st-vec core-type) 0)
         ,(mu:sv-ref (mu:st-vec core-type) 0)))))

(mu:intern :core "%describe-struct"
   (:lambda (struct stream)
      (core:format stream
       "struct: (~A bytes) :struct ~A~%    type:  ~A~%    props: ~A~%"
       `(,(mu:hp-size struct)
         ,(core:type-of struct)
         ,(mu:st-type struct)
         ,(mu:st-vec struct)))))

(mu:intern :core "%describe-symbol"
   (:lambda (symbol stream)
      (:if (core:null symbol)
           (core:format stream
            "symbol: (~A bytes) :null null~%    ns:    ()~%    name:  :nil~%    value: ()~%"
            `(,(mu:hp-size symbol)))
           (:if (core:keywordp symbol)
                (core:format stream
                 "symbol: (~A bytes) :symbol keyword~%    ns:    ()~%    name:  ~A~%    value: ~A~%"
                 `(,(mu:hp-size symbol) ,(mu:sy-name symbol) ,(mu:sy-val symbol)))
                (:if (mu:boundp symbol)
                     ((:lambda (view)
                         (core:format stream
                          "symbol: (~A bytes) :symbol symbol~%    ns:    ~A~%    name:  ~A~%    value: ~A~%"
                          `(,(mu:hp-size symbol)
                            ,(mu:sv-ref view 0)
                            ,(mu:sv-ref view 1)
                            ,(mu:sv-ref view 2))))
                      (mu:view symbol))
                     ((:lambda (view)
                         (core:format stream
                          "symbol: (~A bytes) :symbol symbol~%    ns:      ~A~%    name:    ~A~%    unbound: :t~%"
                          `(,(mu:hp-size symbol)
                            ,(mu:sv-ref view 0)
                            ,(mu:sv-ref view 1)
                            ,(mu:sv-ref view 2))))
                      (mu:view symbol)))))))

(mu:intern :core "%describe-fixnum"
   (:lambda (fx stream)
      (core:format stream "fixnum: (~A bytes) :fixnum ~A~%    format: 61b signed integer~%    value:  ~A~%"
        `(,(mu:hp-size fx) ,(core:type-of fx) ,fx))))

(mu:intern :core "%describe-char"
   (:lambda (ch stream)
      (core:format stream "char: (~A bytes) :char ~A~%    format: 8b ASCII character~%    value:  ~A~%"
        `(,(mu:hp-size ch) ,(core:type-of ch) ,ch))))

(mu:intern :core "%describe-float"
   (:lambda (fl stream)
      (core:format stream
       "float: :float ~A~%    format: 32b IEEE single float~%    value:  ~A~%"
       `(,(core:type-of fl) ,fl))))

(mu:intern :core "%describe-string"
   (:lambda (str stream)
      (core:format stream
       "string: (~A bytes) :vector string~%    format: unsigned 8b character vector~%    length: ~A~%    value:  ~A~%"
       `(,(mu:hp-size str) ,(mu:sv-len str) ,str))))

(mu:intern :core "%describe-vector"
   (:lambda (vec stream)
      (core:format stream
       "vector: (~A bytes) :vector ~A~%    format: ~A~%    length: ~A~%    elements:  ~A~%"
       `(,(mu:hp-size vec) ,(core:type-of vec) ,(mu:sv-type vec) ,(mu:sv-type vec) ,vec))))

(mu:intern :core "%describe-cons"
   (:lambda (cons stream)
      (:if (core:dotted-pair-p cons)
           (core:format stream
            "dotted pair: (~A bytes) :cons ~A~%    value:  ~A~%"
            `(,(mu:hp-size cons) ,(core:type-of cons) ,cons))
           (core:format stream
            "cons: (~A bytes) :cons ~A~%    length: ~A~%    value:  ~A~%"
            `(,(mu:hp-size bytes) ,(core:type-of cons) ,(mu:length cons) ,cons)))))

(mu:intern :core "describe"
   (:lambda (value stream)
      (mu:fix
       (:lambda (list)
          (:if (core:null list)
               ()
               ((:lambda (predicate fn)
                   (:if (mu:apply predicate `(,value))
                        ((:lambda ()
                            (mu:apply fn `(,value ,stream))
                            ()))
                        (mu:cdr list)))
               (mu:car (mu:car list))
               (mu:cdr (mu:car list)))))
       `(,(mu:cons core:functionp core:%describe-function)
         ,(mu:cons core:charp core:%describe-char)
         ,(mu:cons core:core-type-p core:%describe-core-type)
         ,(mu:cons core:structp core:%describe-struct)
         ,(mu:cons core:symbolp core:%describe-symbol)
         ,(mu:cons core:fixnump core:%describe-fixnum)
         ,(mu:cons core:floatp core:%describe-float)
         ,(mu:cons core:stringp core:%describe-string)
         ,(mu:cons core:vectorp core:%describe-vector)
          ,(mu:cons core:consp core:%describe-cons)))))
