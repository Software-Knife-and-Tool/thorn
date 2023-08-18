;;;  Spdx-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; debugging
;;;
(mu:untern core :intern "debug-core-symbols")

(mu:intern core :intern "describe-type-table"
   (:lambda (obj)
      (mu:cdr (core:assoc
        (core:type-of obj)
        '((:func . core::describe-function)

          )))))

(mu:intern core :intern "describe-function"
   (:lambda (fn)
      ((:lambda (view)
          (core:format :t "simple function: [~A] arity: ~A body ~A~%"
                       `(,(mu:sv-ref view 2)
                         ,(mu:sv-ref view 0)
                         ,(mu:sv-ref view 1))))
       (mu:view fn))))

(mu:intern core :intern "describe"
   (:lambda (obj)
      (mu:fix
       (:lambda (list)
          (:if (core:null list)
               ()
               ((:lambda (predicate fn)
                   (:if (mu:apply predicate obj)
                        ((:lambda ()
                            (mu:apply fn obj)
                            ()))
                        (mu:cdr list)))
               (mu:car (mu:car list))
               (mu:cdr (mu:car list))))
          `(,(mu:cons core:functionp core::describe-function))))))

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
                            ((:lambda (form)
                                (core:format :t "[~A] ~S~%" `(,(mu:type-of form) ,form))
                                (core:null loop))
                             (mu:eval (core:compile form)))))
                    (core:read :t () ()))))))
       ())))
