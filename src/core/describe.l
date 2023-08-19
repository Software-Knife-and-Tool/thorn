;;;  Spdx-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; describe
;;;
(mu:intern core :intern "describe-function"
   (:lambda (fn)
      ((:lambda (view)
          (core:format :t
            "function: :func ~A~%    name:  [~A]~%    arity: ~A~%    body:  ~A~%"
            `(,(core:type-of fn) 
              ,(mu:sv-ref view 2)
              ,(mu:sv-ref view 0)
              ,(mu:sv-ref view 1))))
       (mu:view fn))))

(mu:intern core :intern "describe-core-type"
   (:lambda (core-type)
      (core:format :t
       "core-type: :<ctype> ~A~%    type:  ~A~%    props: ~A~%"       
       `(,(core:type-of core-type)
         ,(mu:sv-ref (mu:st-vec core-type) 0)
         ,(mu:sv-ref (mu:st-vec core-type) 0)))))

(mu:intern core :intern "describe-struct"
   (:lambda (struct)
      (core:format :t
       "closure: :struct ~A~%    type:  ~A~%    props: ~A~%"       
       `(,(core:type-of struct)
         ,(mu:st-type struct)
         ,(mu:st-vec struct)))))

(mu:intern core :intern "describe-closure"
   (:lambda (closure)
      (core:format :t
       "closure: :struct ~A~%    arity: ~A~%    fn:    ~A~%    env:  ~A~%"       
       `(,(core:type-of closure)
         ,(core::closure-prop :nargs closure)
         ,(core::closure-prop :fn closure)
         ,(core::closure-prop :env closure)))))

(mu:intern core :intern "describe-symbol"
   (:lambda (symbol)
      (:if (core:null symbol) 
           (core:format :t
            "symbol: :null null~%    ns:    ()~%    scope: :extern~%    name:  :nil~%    value: ()~%"
            ())
           (:if (core:keywordp symbol)
                (core:format :t
                 "symbol: :symbol keyword~%    ns:    ()~%    scope: :extern~%    name:  ~A~%    value: ~A~%"
                 `(,(mu:sy-name symbol) ,(mu:sy-val symbol)))
                (:if (mu:boundp symbol)
                     ((:lambda (view)
                         (core:format :t
                          "symbol: :symbol symbol~%    ns:    ~A~%    scope: ~A~%    name:  ~A~%    value: ~A~%"
                          `(,(mu:sv-ref view 0)
                            ,(mu:sv-ref view 1)
                            ,(mu:sv-ref view 2)
                            ,(mu:sv-ref view 3))))
                      (mu:view symbol))
                     ((:lambda (view)
                         (core:format :t
                          "symbol: :symbol symbol~%    ns:      ~A~%    scope:   ~A~%    name:    ~A~%    unbound: :t~%"
                          `(,(mu:sv-ref view 0)
                            ,(mu:sv-ref view 1)
                            ,(mu:sv-ref view 2))))
                      (mu:view symbol)))))))

(mu:intern core :intern "describe-fixnum"
   (:lambda (fx)
      (core:format :t "fixnum: :fixnum ~A~%    format: 61b signed integer~%    value:  ~A~%"
        `(,(core:type-of fx) ,fx))))

(mu:intern core :intern "describe-float"
   (:lambda (fl)
      (core:format :t
       "float: :float ~A~%    format: 32b IEEE single float~%    value:  ~A~%"
       `(,(core:type-of fl) ,fl))))

(mu:intern core :intern "describe-string"
   (:lambda (str)
      (core:format :t
       "string: :vector string~%    format: unsigned 8bit character vector~%    length: ~A~%    value:  ~A~%"
       `(,(mu:sv-len str) ,str))))

(mu:intern core :intern "describe-vector"
   (:lambda (vec)
      (core:format :t
       "string: :vector ~A~%    format: ~A~%    length: ~A~%    value:  ~A~%"
       `(,(core:type-of vec) ,(mu:sv-type vec) ,(mu:sv-type vec) ,vec))))

(mu:intern core :intern "describe-cons"
   (:lambda (cons)
      (:if (core:dotted-pair-p cons)
           (core:format :t
            "dotted pair: :cons ~A~%    value:  ~A~%"
            `(,(core:type-of cons) ,cons))
           (core:format :t
            "cons: :cons ~A~%    length: ~A~%    value:  ~A~%"
            `(,(core:type-of cons) ,(mu:length cons) ,cons)))))

(mu:intern core :intern "describe"
   (:lambda (obj)
      (mu:fix
       (:lambda (list)
          (:if (core:null list)
               ()
               ((:lambda (predicate fn)
                   (:if (mu:apply predicate `(,obj))
                        ((:lambda ()
                            (mu:apply fn `(,obj))
                            ()))
                        (mu:cdr list)))
               (mu:car (mu:car list))
               (mu:cdr (mu:car list)))))
       `(,(mu:cons core:functionp core::describe-function)
         ,(mu:cons core:closurep core::describe-closure)
         ,(mu:cons core:core-type-p core::describe-core-type)
         ,(mu:cons core:structp core::describe-struct)
         ,(mu:cons core:symbolp core::describe-symbol)
         ,(mu:cons core:fixnump core::describe-fixnum)
         ,(mu:cons core:floatp core::describe-float)
         ,(mu:cons core:stringp core::describe-string)
         ,(mu:cons core:vectorp core::describe-vector)
         ,(mu:cons core:consp core::describe-cons)))))
