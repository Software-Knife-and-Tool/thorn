;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; core parsers
;;;

;;;
;;; number reader
;;;
(mu:intern core "parse-integer"
  (:lambda (digits base)
    (core:raise-unless core:stringp digits 'core:parse-integer "not a string")
    (core:raise-unless core:fixnump base 'core:parse-integer "not a fixnum")
    (:if (core:zerop (core:length digits))
         ()
         ((:lambda (sign)
            ((:lambda (integer)
               (:if (core:fixnump integer)
                    (:if (core:null sign)
                         integer
                         (mu:fx-mul sign integer))
                    ()))
             (mu:cdr
              (mu:fix
               (:lambda (loop)
                 (:if (core:null loop)
                      ()
                      ((:lambda (index acc)
                         (:if (mu:fx-lt (core:1- (core:length digits)) index)
                              loop
                              ((:lambda (n)
                                 (:if (core:null n)
                                      ()
                                      (:if (core:%orf (core:null n) (mu:fx-lt (core:1- base) n))
                                           ()
                                           (mu:cons (core:1+ index) (mu:fx-add n (mu:fx-mul acc base))))))
                               (core:string-position (core:schar digits index) "0123456789abcdef"))))
                       (mu:car loop)
                       (mu:cdr loop))))
               (:if (core:fixnump sign)
                    '(1 . 0)
                    '(0 . 0))))))
          ((:lambda (digit)
             (:if (mu:eq #\- digit)
                  -1
                  (:if (mu:eq #\+ digit)
                       1
                       ())))
           (core:schar digits 0))))))

(mu:intern core "parse-float"
  (:lambda (str)
    ((:lambda (stream)
       ((:lambda (float)
          (:if (core:floatp float)
               float
               ()))
          (mu:read stream () ())))
       (mu:open :string :input str))))
