;;;  Spdx-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; debugging
;;;
(mu:untern core :intern "debug-core-symbols")

(mu:intern core :intern "describe-function"
   (:lambda (fn)
      ((:lambda (view)
          (core:format :t "function: types: :func func~%    name:  [~A]~%    arity: ~A~%    body:  ~A~%"
                       `(,(mu:sv-ref view 2)
                         ,(mu:sv-ref view 0)
                         ,(mu:sv-ref view 1))))
       (mu:view fn))))

(mu:intern core :intern "describe-closure"
   (:lambda (closure)
      (core:format
       :t
       "closure: types: :struct closure~%    arity: ~A~%    fn:    ~A~%    env:  ~A~%"       
       `(,(core::closure-prop :nargs closure)
         ,(core::closure-prop :fn closure)
         ,(core::closure-prop :env closure)))))

(mu:intern core :intern "describe-symbol"
   (:lambda (symbol)
      (:if (core:null symbol) 
           (core:format
            :t
            "symbol: types: :null null~%    ns:    ()~%    scope: :extern~%    name:  :nil~%    value: ()~%"
            ())
           (:if (core:keywordp symbol)
                (core:format
                 :t
                 "symbol: types: :null null~%    ns:    ()~%    scope: :extern~%    name:  ~A~%    value: ~A~%"
                 `(,(mu:sy-name symbol) ,(mu:sy-val symbol)))
                ((:lambda (view)
                    (core:format
                     :t
                     "symbol: types: :symbol symbol~%    ns:    ~A~%    scope: ~A~%    name:  ~A~%    value: ~A~%"
                     `(,(mu:sv-ref view 0)
                       ,(mu:sv-ref view 1)
                       ,(mu:sv-ref view 2)
                       ,(mu:sv-ref view 3))))
                 (mu:view symbol))))))

(mu:intern core :intern "describe-fixnum"
   (:lambda (fx)
      (core:format :t "fixnum: types: :fixnum fixnum~%    format: 61b signed integer~%    value:  ~A~%" `(,fx))))

(mu:intern core :intern "describe-float"
   (:lambda (fl)
      (core:format
       :t
       "float: types: :float float~%    format: 32b IEEE single float~%    value:  ~A~%"
       `(,fl))))

(mu:intern core :intern "describe-string"
   (:lambda (str)
      (core:format
       :t
       "string: types: :vector string~%    format: unsigned 8bit character vector~%    length: ~A~%    value:  ~A~%"
       `(,(mu:sv-len str) ,str))))

(mu:intern core :intern "describe-vector"
   (:lambda (vec)
      (core:format
       :t
       "string: types: :vector vector~%    format: ~A~%    length: ~A~%    value:  ~A~%"
       `(,(mu:sv-type vec) ,(mu:sv-type vec) ,vec))))

(mu:intern core :intern "describe-cons"
   (:lambda (cons)
      (:if (core:dotted-pair-p cons)
           (core:format
            :t
            "dotted pair: types: :cons cons~%    value:  ~A~%"
            `(,cons))
           (core:format
            :t
            "cons: types: :cons cons~%    length: ~A~%    value:  ~A~%"
            `(,(mu:length cons) ,cons)))))

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
         ,(mu:cons core:symbolp core::describe-symbol)
         ,(mu:cons core:fixnump core::describe-fixnum)
         ,(mu:cons core:floatp core::describe-float)
         ,(mu:cons core:stringp core::describe-string)
         ,(mu:cons core:vectorp core::describe-vector)
         ,(mu:cons core:consp core::describe-cons)))))

(mu:intern core :intern "debug-map-symbol"
   (:lambda (obj)
      (:if (mu:boundp 'core::debug-core-symbols)
           ()
           (mu:intern core :intern "debug-core-symbols" `(,(mu:ns-ext core) ,(mu:ns-int core))))
      (core:mapcar
       (:lambda (scope)
          (mu:fix
           (:lambda (list)
              (:if (core:consp list)
                   ((:lambda (symbol)
                       (:if (mu:eq (mu:sy-val symbol) obj)
                            symbol
                            (mu:cdr list)))
                    (mu:car list))
                   list))
           scope))
       core::debug-core-symbols)))

(mu:intern core :intern "repl"
   (:lambda ()
      (mu:fix
       (:lambda (loop)
          (:if (core:eof :t)
               loop
               (core:with-exception
                (:lambda (ex)
                   (core:exceptf mu:err-out "exception: ~A on ~A by ~S from ~A~%" :t ex)
                   (core:null loop))
                (:lambda ()
                   (core:format :t "core> " ())
                   (mu:flush mu:std-out)
                   ((:lambda (form)
                       (:if (core:eof :t)
                            loop
                            ((:lambda (value)
                                (core:format :t "[~A] ~S~%" `(,(mu:type-of value) ,value))
                                (core:null loop))
                             (mu:eval (core:compile form)))))
                    (core:read :t () ()))))))
       ())))
