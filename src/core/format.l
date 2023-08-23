;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; core format
;;;
(mu:intern core "%fmt-tilde"
  (:lambda (dest args)
    (core:write-char #\~ dest)
    args))

(mu:intern core "%fmt-tab"
  (:lambda (dest args)
    (core:write-char #\tab dest)
    args))

(mu:intern core "%fmt-percent"
  (:lambda (dest args)
    (core:terpri dest)
    args))

(mu:intern core "%fmt-bang"
  (:lambda (dest args)
    (mu:cdr args)))

(mu:intern core "%fmt-decimal"
  (:lambda (dest args)
    (core:raise-unless core:fixnump (mu:car args) 'core:format "argument to ~D not a fixnum")
    (core:write (mu:car args) () dest)
    (mu:cdr args)))

(mu:intern core "%fmt-hex"
  (:lambda (dest args)
    ((:lambda (fx)
       (core:raise-unless core:fixnump fx 'core:format "argument to ~X not a fixnum")
       (:if (core:zerop fx)
            (core:write "0" () dest)
            ((:lambda (str-stream)
               (mu:fix
                (:lambda (n)
                  (:if (core:%orf (core:null n) (core:zerop n))
                       ()
                       (core:%prog2
                           (mu:write (core:schar "0123456789abcdef" (mu:logand n #xf)) () str-stream)
                           (mu:fx-div n 16))))
                (:if (mu:fx-lt fx 0) (mu:fx-mul -1 fx) fx))
               (:if (mu:fx-lt fx 0)
                    (mu:write #\- () str-stream)
                    ())
               ((:lambda (hexstr)
                  (mu:fix
                   (:lambda (len)
                     (:if (core:zerop len)
                          0
                          (core:%prog2
                              (mu:write (core:schar hexstr (core:1- len)) () dest)
                              (core:1- len))))
                   (mu:sv-len hexstr)))
                  (mu:get-str str-stream)))
            (mu:open :string :output ""))))
       (mu:car args))))

(mu:intern core "%fmt-aesthetic"
  (:lambda (dest args)
    (core:write (mu:car args) () dest)
    (mu:cdr args)))

(mu:intern core "%fmt-standard"
  (:lambda (dest args)
    (core:write (mu:car args) :t dest)
    (mu:cdr args)))

;;;
;;; control ops
;;;
(mu:intern core "%fmt-op"
  (:lambda (dir dest args)
    ((:lambda (fn)
       (:if fn
            (core:apply (mu:sy-val fn) `(,dest ,args))
            (core:raise dir 'core:format "unrecognized format directive")))
     (mu:cdr
      (core:assoc
       dir
       '((#\A . core:%fmt-aesthetic)
         (#\S . core:%fmt-standard)
         (#\X . core:%fmt-hex)
         (#\D . core:%fmt-decimal)
         (#\T . core:%fmt-tab)
         (#\~ . core:%fmt-tilde)
         (#\% . core:%fmt-percent)
         (#\! . core:%fmt-bang)))))))

(mu:intern core "%fmt-loop"
  (:lambda (stream dest fmt args)
    (mu:fix
     (:lambda (loop)
       ((:lambda (nth argl)
          (:if (mu:eq nth (core:length fmt))
               loop
               ((:lambda (ch)
                  (:if (mu:eq ch #\~)
                       (core:%prog2
                           (core:raise-if (mu:eq (core:1+ nth) (core:length fmt)) nth 'core:format "unexpected end of string")
                           (mu:cons (mu:fx-add 2 nth) (core:%fmt-op (core:schar fmt (core:1+ nth)) dest argl)))
                       (core:%prog2
                           (core:write-char ch dest)
                           (mu:cons (core:1+ nth) argl))))
                (core:schar fmt nth))))
        (mu:car loop)
        (mu:cdr loop)))
     (mu:cons 0 args))
    (:if (core:null stream)
         (core:get-output-stream-string dest)
         ())))

(mu:intern core "format"
   (:lambda (stream fmt args)
     (core:raise-unless core:stringp fmt 'core:format "not a string")
     (core:raise-unless core:listp args 'core:format "not a list")
     (:if (core:zerop (core:length fmt))
          (:if stream
               ""
               ())
          ((:lambda (dest) (core:%fmt-loop stream dest fmt args))
           (:if (core:null stream)
                (mu:open :string :output "")
                (:if (mu:eq stream :t)
                     mu:std-out
                     (:if (core:streamp stream)
                          stream
                          (core:raise stream 'core:format "not a stream designator"))))))))
