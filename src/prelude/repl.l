;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; repl
;;;
(mu:intern :prelude "repl"
   (:lambda ()
     (mu:fix
       (:lambda (loop)
          (:if (prelude:eof :t)
               loop
               (prelude:with-exception
                (:lambda (ex)
                   (prelude:break ex)
                   (prelude:null loop))
                (:lambda ()
                   (prelude:format :t "prelude> " ())
                   (mu:flush mu:std-out)
                   ((:lambda (form)
                       (:if (prelude:eof :t)
                            loop
                            ((:lambda (value)
                                (prelude:format :t "[~A] ~S~%" `(,(prelude:type-of value) ,value))
                                (prelude:null loop))
                             (mu:eval (prelude:compile form)))))
                    (prelude:read :t () ()))))))
       ())))
