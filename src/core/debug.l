;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; debug repl
;;;
(mu:intern :core "%describe-function"
   (:lambda (fn)
      ((:lambda (view)
          (core:format :t
            "function: :func ~A~%    name:  [~A]~%    arity: ~A~%    body:  ~A~%"
            `(,(core:type-of fn)
              ,(mu:sv-ref view 2)
              ,(mu:sv-ref view 0)
              ,(mu:sv-ref view 1))))
       (mu:view fn))))

(mu:intern :core "%describe-core-type"
   (:lambda (core-type)
      (core:format :t
       "core-type: :<ctype> ~A~%    type:  ~A~%    props: ~A~%"
       `(,(core:type-of core-type)
         ,(mu:sv-ref (mu:st-vec core-type) 0)
         ,(mu:sv-ref (mu:st-vec core-type) 0)))))

(mu:intern :core "%describe-struct"
   (:lambda (struct)
      (core:format :t
       "funct: :struct ~A~%    type:  ~A~%    props: ~A~%"
       `(,(core:type-of struct)
         ,(mu:st-type struct)
         ,(mu:st-vec struct)))))

(mu:intern :core "%describe-function"
   (:lambda (fn)
      (core:format :t
       "function: :struct ~A~%    arity: ~A~%    fn:    ~A~%    env:  ~A~%"
       `(,(core:type-of gn)
         ,(core:%function-prop :nargs fn)
         ,(core:%function-prop :fn fn)
         ,(core:%function-prop :env fn)))))

(mu:intern :core "%describe-symbol"
   (:lambda (symbol)
      (:if (core:null symbol)
           (core:format :t
            "symbol: :null null~%    ns:    ()~%    name:  :nil~%    value: ()~%"
            ())
           (:if (core:keywordp symbol)
                (core:format :t
                 "symbol: :symbol keyword~%    ns:    ()~%    name:  ~A~%    value: ~A~%"
                 `(,(mu:sy-name symbol) ,(mu:sy-val symbol)))
                (:if (mu:boundp symbol)
                     ((:lambda (view)
                         (core:format :t
                          "symbol: :symbol symbol~%    ns:    ~A~%    name:  ~A~%    value: ~A~%"
                          `(,(mu:sv-ref view 0)
                            ,(mu:sv-ref view 1)
                            ,(mu:sv-ref view 2))))
                      (mu:view symbol))
                     ((:lambda (view)
                         (core:format :t
                          "symbol: :symbol symbol~%    ns:      ~A~%    name:    ~A~%    unbound: :t~%"
                          `(,(mu:sv-ref view 0)
                            ,(mu:sv-ref view 1)
                            ,(mu:sv-ref view 2))))
                      (mu:view symbol)))))))

(mu:intern :core "%describe-fixnum"
   (:lambda (fx)
      (core:format :t "fixnum: :fixnum ~A~%    format: 61b signed integer~%    value:  ~A~%"
        `(,(core:type-of fx) ,fx))))

(mu:intern :core "%describe-float"
   (:lambda (fl)
      (core:format :t
       "float: :float ~A~%    format: 32b IEEE single float~%    value:  ~A~%"
       `(,(core:type-of fl) ,fl))))

(mu:intern :core "%describe-string"
   (:lambda (str)
      (core:format :t
       "string: :vector string~%    format: unsigned 8bit character vector~%    length: ~A~%    value:  ~A~%"
       `(,(mu:sv-len str) ,str))))

(mu:intern :core "%describe-vector"
   (:lambda (vec)
      (core:format :t
       "string: :vector ~A~%    format: ~A~%    length: ~A~%    value:  ~A~%"
       `(,(core:type-of vec) ,(mu:sv-type vec) ,(mu:sv-type vec) ,vec))))

(mu:intern :core "%describe-cons"
   (:lambda (cons)
      (:if (core:dotted-pair-p cons)
           (core:format :t
            "dotted pair: :cons ~A~%    value:  ~A~%"
            `(,(core:type-of cons) ,cons))
           (core:format :t
            "cons: :cons ~A~%    length: ~A~%    value:  ~A~%"
            `(,(core:type-of cons) ,(mu:length cons) ,cons)))))

(mu:intern :core "describe"
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
       `(,(mu:cons core:functionp core:%describe-function)
         ,(mu:cons core:core-type-p core:%describe-core-type)
         ,(mu:cons core:structp core:%describe-struct)
         ,(mu:cons core:symbolp core:%describe-symbol)
         ,(mu:cons core:fixnump core:%describe-fixnum)
         ,(mu:cons core:floatp core:%describe-float)
         ,(mu:cons core:stringp core:%describe-string)
         ,(mu:cons core:vectorp core:%describe-vector)
         ,(mu:cons core:consp core:%describe-cons)))))

(mu:intern :core "debugger"
   (:lambda (except)
     (core:format :t ";;; entering break loop with exception~%" ())
     (core:exceptf mu:std-out ";;; ~A on ~A by ~S, ~A~%" () except)
     (core:format :t ";;; :h for commands~%" ())
     (mu:fix
      (:lambda (loop)
         (:if (core:eof :t)
              ()
              ((:lambda (form)
                  (:if (mu:eq form :h)
                       ((:lambda ()
                           (core:format :t "debugger help:~%" ())
                           (core:format :t ":e - print the full exception~%" ())
                           (core:format :t ":r - return from break~%" ())
                           (core:format :t ":x - exit process~%" ())
                           (core:null loop)))
                       (:if (mu:eq form :x)
                            (mu:exit 0)
                            (:if (mu:eq form :e)
                                 ((:lambda ()
                                     (core:exceptf mu:std-out "exception: ~A on ~A by ~S, ~A~%" :t except)
                                     (core:null loop)))
                                 (:if (mu:eq form :r)
                                      loop
                                      ((:lambda ()
                                          (core:format :t ";;; unrecognized debugger command: h for help~%" ()))
                                          (core:null loop)))))))
                   ((:lambda ()
                       (core:format :t "core:debug> " ())
                       (mu:flush mu:std-out)
                       (core:read :t () ()))))))
     ())))

(mu:intern :core "repl"
   (:lambda ()
     (mu:fix
       (:lambda (loop)
          (:if (core:eof :t)
               loop
               (core:with-exception
                (:lambda (ex)
                   (core:debugger ex)
                   (core:null loop))
                (:lambda ()
                   (core:format :t "core> " ())
                   (mu:flush mu:std-out)
                   ((:lambda (form)
                       (:if (core:eof :t)
                            loop
                            ((:lambda (value)
                                (core:format :t "[~A] ~S~%" `(,(core:type-of value) ,value))
                                (core:null loop))
                             (mu:eval (core:compile form)))))
                    (core:read :t () ()))))))
       ())))
