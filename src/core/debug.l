;;;  Spdx-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; debugging
;;;
(mu:intern core :intern "describe"
   (:lambda (obj)))

(mu:untern core :intern "core-symbols")

(mu:intern core :intern "lookup-symbol-by-value"
   (:lambda (obj)
      (:if (mu:unboundp 'core::core-symbols)
           (mu:intern core :intern "symbols" `(,(mu:ns-ext core) ,(mu:ns-int core)))
           ())
      (mu:fix
       (:lambda (list)
          (:if (core:consp list)
               ((:lambda (symbol)
                   (:if (mu:eq (mu:sy-val symbol) obj)
                        symbol
                        (mu:cdr list)))
                (mu:car list))
               list))
       (mu:car core::core-symbols))))

(mu:intern core :intern "repl"
   (:lambda ()
      (mu:fix
       (:lambda (loop)
          (:if (core:eof :t)
               loop
               (core:with-exception
                (:lambda (ex)
                   (core:exceptionf mu:err-out "exception: ~A on ~A by ~S from ~A~%" :t ex)
                   (core:null loop))
                (:lambda ()
                   (core:format :t "core> " ())
                   (mu:flush mu:std-out)
                   ((:lambda (form)
                       (:if (core:eof :t)
                            ()
                            ((:lambda (form)
                                (core:format :t "[~A] ~S~%" `(,(mu:type-of form) ,form))
                                (core:null loop))
                             (mu:eval (core:compile form)))))
                    (core:read :t () ()))))))
       ())))
