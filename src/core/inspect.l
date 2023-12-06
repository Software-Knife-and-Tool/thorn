;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; inspect
;;;
(mu:intern :core "%inspect-function"
   (:lambda (obj)
      ((:lambda (view)
          ((:lambda (tags)
             (mu:make-st :inspect `(function ,(mu:hp-size obj) ,tags)))
           `(,(mu:cons :arity (mu:sv-ref view 0))
             ,(mu:cons :body (mu:sv-ref view 1)))))
       (mu:view obj))))

(mu:intern :core "%inspect-core-type"
   (:lambda (obj)
     ((:lambda (view)
         ((:lambda (tags)
            (mu:make-st :inspect `(:<ctype> ,(mu:hp-size obj) ,tags)))
          `(,(mu:cons :type (mu:sv-ref (mu:st-vec obj) 0))
            ,(mu:cons :values (mu:sv-ref (mu:st-vec obj) 1)))))
      (mu:view obj))))

(mu:intern :core "%inspect-struct"
   (:lambda (obj)
     ((:lambda (view)
         ((:lambda (tags)  
            (mu:make-st :inspect `(struct ,(mu:hp-size obj) ,tags)))
          `(,(mu:cons :type (mu:st-type obj))
            ,(mu:cons :values (mu:st-vec obj)))))
      (mu:view obj))))

(mu:intern :core "%inspect-symbol"
   (:lambda (obj)
     ((:lambda (view)
         ((:lambda (tags)  
            (mu:make-st :inspect `(symbol ,(mu:hp-size obj) ,tags)))
          `(,(mu:cons :ns (mu:sv-ref view 0))
            ,(mu:cons :name (mu:sv-ref view 1))
            ,(mu:cons :value (mu:sv-ref view 2)))))
      (mu:view symbol))))

(mu:intern :core "%inspect-const"
   (:lambda (obj)
     (mu:make-st :inspect `(,(core:type-of obj) ,(mu:hp-size obj) ()))))

(mu:intern :core "%inspect-vector"
   (:lambda (obj)
     ((:lambda (tags)          
         (mu:make-st :inspect `(vector ,(mu:hp-size obj) ,tags)))
      `(,(mu:cons :type (mu:sv-type obj))
        ,(mu:cons :length (mu:sv-len obj))))))

(mu:intern :core "%inspect-cons"
   (:lambda (obj)
     ((:lambda (tags)          
         (mu:make-st :inspect `(cons ,(mu:hp-size obj) ,tags)))
      `(,(mu:cons :car (mu:car obj))
        ,(mu:cons :cdr (mu:cdr obj))))))

(mu:intern :core "%inspect"
   (:lambda (value)
      (mu:fix
       (:lambda (loop)
          (:if (core:structp loop)
               loop
               (:if (core:null loop)
                    (mu:make-st :inspect `(unknown 0 ()))
                    ((:lambda (predicate fn)
                       (:if (mu:apply predicate `(,value))
                            (mu:apply fn `(,value))
                            (mu:cdr loop)))
                     (mu:car (mu:car loop))
                     (mu:cdr (mu:car loop))))))
       `(,(mu:cons core:core-type-p core:%inspect-core-type)
         ,(mu:cons core:charp core:%inspect-const)
         ,(mu:cons core:consp core:%inspect-cons)
         ,(mu:cons core:fixnump core:%inspect-const)
         ,(mu:cons core:floatp core:%inspect-const)
         ,(mu:cons core:functionp core:%inspect-function)
         ,(mu:cons core:structp core:%inspect-struct)
         ,(mu:cons core:symbolp core:%inspect-symbol)
         ,(mu:cons core:vectorp core:%inspect-vector)))))
