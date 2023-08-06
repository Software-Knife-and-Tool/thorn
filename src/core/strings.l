
;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; strings
;;;
(mu:intern core :extern "stringp"
   (:lambda (string)
      (:if (core:vectorp string)
           (mu:eq :char (mu:sv-type string))
           ())))

(mu:intern core :extern "get-output-stream-string"
   (:lambda (stream)
      (mu:get-str stream)))

(mu:intern core :extern "schar"
  (:lambda (str index)
    (core:raise-unless core:stringp str 'core:schar "not a string")
    (core:raise-unless core:fixnump index 'core:schar "not a string")
    (mu:sv-ref str index)))

(mu:intern core :extern "string="
   (:lambda (str-1 str-2)
      (core:raise-unless core:stringp str-1 'core:string= "not a string")
      (core:raise-unless core:stringp str-2 'core:string= "not a string")
      (:if (mu:eq str-1 str-2)
         :t
         (:if (mu:eq (mu:sv-len str-1) (mu:sv-len str-2))
              (mu:fix
               (:lambda (nth)
                 (:if (core:numberp nth)
                      (:if (mu:eq nth (mu:sv-len str-1))
                           :t
                           (:if (mu:eq (mu:sv-ref str-1 nth) (mu:sv-ref str-2 nth))
                                (core:1+ nth)
                                ()))
                      nth))
               0)
              ()))))

;;;
;;; string-position
;;;
(mu:intern core :extern "string-position"
  (:lambda (ch str)
    (core:raise-unless core:charp ch 'core:string-position "not a char")
    (core:raise-unless core:stringp str 'core:string-position "not a string")
     (:if (core:zerop (core:length str))
          ()
          ((:lambda (len)
             (mu:fix
              (:lambda (loop)
                (:if (core:numberp loop)
                     (:if (mu:fx-lt (core:1- len) loop)
                          ()
                          (:if (mu:eq ch (mu:sv-ref str loop))
                               loop
                               (core:1+ loop)))
                     loop))
             0))
           (core:length str)))))

;;;
;;; string-find
;;;
(mu:intern core :extern "string-find"
  (:lambda (ch str)
    (core:raise-unless core:charp ch 'core:string-find "not a char")
    (core:raise-unless core:stringp str 'core:string-find "not a string")
    (:if (core:zerop (core:length str))
         ()
         ((:lambda (len)
            (mu:fix
             (:lambda (loop)
               (:if (core:numberp loop)
                    (:if (mu:fx-lt (core:1- len) loop)
                         ()
                         (:if (mu:eq ch (mu:sv-ref str loop))
                              ch
                              (core:1+ loop)))
                    loop))
             0))
          (core:length str)))))

;;;
;;; string construction
;;;
(mu:intern core :extern "string"
   (:lambda (designator)
      (:if (core:stringp designator)
         designator
         (:if (mu:eq :symbol (mu:type-of designator))
              (mu:sy-name designator)
              (:if (core:charp designator)
                   (mu:make-sv :char `(,designator))
                   (core:raise designator 'core:string "not a string designator"))))))

(mu:intern core :extern "string-append"
  (:lambda (list)
    (core:raise-unless core:listp list 'core:string "is not a list")
    ((:lambda (stream)
       (mu:fix
        (:lambda (list)
          (:if list
               (core::prog2
                   ((:lambda (str)
                      (core:raise-unless core:stringp str 'core:string-append "is not a string")
                      (mu:write str () stream))
                    (mu:car list))
                   (mu:cdr list))
               ()))
        list)
       (mu:get-str stream))
     (mu:open :string :output ""))))

(mu:intern core :extern "substr"
  (:lambda (str start end)
     (core:raise-unless core:stringp str 'core:substr "is not a string")
     (core:raise-unless core:fixnump start 'core:substr "start is not a fixnum")
     (core:raise-unless core:fixnump end 'core:substr "end is not a fixnum")
     ((:lambda (substr)
        (mu:fix
         (:lambda (nth)
           (:if (core:numberp nth)
                (:if (mu:eq nth (mu:sv-len str))
                     ()
                     (:if (mu:fx-lt nth (core:1+ end))
                          (core::prog2
                              (mu:write (mu:sv-ref str nth) () substr)
                              (core:1+ nth))
                          ()))
                nth))
         start)
        (mu:get-str substr))
        (mu:open :string :output ""))))
