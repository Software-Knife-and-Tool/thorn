;;;  Spdx-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; debugging
;;;
(mu:intern core::ns :intern "describe"
   (:lambda (obj)))

(mu:intern core::ns :intern "repl"
   (:lambda ()
      (mu:fix
       (:lambda (loop)
          (:if (core:eof :t)
               loop
               (core:with-exception
                (:lambda (ex)
                   (core:exceptionf mu:err-out "exception: condition: ~A on ~A by ~S from ~A~%" () ex)
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
