;;;  Spdx-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; repl
;;;
(mu:intern core "repl"
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
                                (core:format :t "[~A] ~S~%" `(,(core:type-of value) ,value))
                                (core:null loop))
                             (mu:eval (core:compile form)))))
                    (core:read :t () ()))))))
       ())))
