;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; strings
;;;
(mu:intern core "stringp"
   (:lambda (string)
      (:if (core:vectorp string)
           (mu:eq :char (mu:sv-type string))
           ())))

(mu:intern core "get-output-stream-string"
   (:lambda (stream)
      (mu:get-str stream)))

(mu:intern core "schar"
  (:lambda (str index)
     (:if (core:stringp str)
          (:if (core:fixnump index)
               (mu:sv-ref str index)
               (core:raise index 'core:schar "not a string"))
          (core:raise str 'core:schar "not a string"))))

(mu:intern core "string="
   (:lambda (str-1 str-2)
      (:if (core:stringp str-1)
           (:if (core:stringp str-2)
                (:if (mu:eq str-1 str-2)
                     :t
                     (:if (mu:eq (mu:sv-len str-1) (mu:sv-len str-2))
                          (mu:fix
                           (:lambda (nth)
                              (:if (core:numberp nth)
                                   (:if (mu:eq nth (mu:sv-len str-1))
                                        :t
                                        (:if (mu:eq (mu:sv-ref str-1 nth) (mu:sv-ref str-2 nth))
                                             (mu:fx-add 1 nth)
                                             ()))
                                   nth))
                           0)
                          ()))
                (core:raise str-2 'core:string= "not a string"))
           (core:raise str-1 'core:string= "not a string"))))

;;;
;;; string-position
;;;
(mu:intern core "string-position"
  (:lambda (ch str)
    (:if (core:charp ch)
         (:if (core:stringp str)
              (:if (core:zerop (core:length str))
                   ()
                   ((:lambda (len)
                       (mu:fix
                        (:lambda (loop)
                           (:if (core:numberp loop)
                                (:if (mu:fx-lt (mu:fx-sub len 1) loop)
                                     ()
                                     (:if (mu:eq ch (mu:sv-ref str loop))
                                          loop
                                          (mu:fx-add 1 loop)))
                                loop))
                        0))
                    (core:length str)))
              (core:raise str 'core:string-position "not a string"))
         (core:raise ch 'core:string-position "not a char"))))

;;;
;;; string-find
;;;
(mu:intern core "string-find"
  (:lambda (ch str)
    (:if (core:charp ch)
         (:if (core:stringp str)
              (:if (core:zerop (core:length str))
                   ()
                   ((:lambda (len)
                       (mu:fix
                        (:lambda (loop)
                           (:if (core:numberp loop)
                                (:if (mu:fx-lt (mu:fx-sub len 1) loop)
                                     ()
                                     (:if (mu:eq ch (mu:sv-ref str loop))
                                          ch
                                          (mu:fx-add 1 loop)))
                                loop))
                        0))
                    (core:length str)))
              (core:raise str 'core:string-find "not a string"))
         (core:raise ch 'core:string-find "not a char"))))

;;;
;;; string construction
;;;
(mu:intern core "string"
   (:lambda (designator)
      (:if (core:stringp designator)
         designator
         (:if (mu:eq :symbol (mu:type-of designator))
              (mu:sy-name designator)
              (:if (core:charp designator)
                   (mu:make-sv :char `(,designator))
                   (core:raise designator 'core:string "not a string designator"))))))

(mu:intern core "string-append"
   (:lambda (list)
      (:if (core:listp list) 
           ((:lambda (stream)
               (mu:fix
                (:lambda (list)
                   (:if list
                        ((:lambda ()
                            ((:lambda (str)
                                (:if (core:stringp str)
                                     (mu:write str () stream)
                                     (core:raise str 'core:string-append "is not a string")))
                             (mu:car list))
                            (mu:cdr list)))
                        ()))
                list)
               (mu:get-str stream))
            (mu:open :string :output ""))
           (core:raise list 'core:string "is not a list"))))

(mu:intern core "substr"
  (:lambda (str start end)
     (:if (core:stringp str)
          (:if (core:fixnump start)
               (:if (core:fixnump end)
                    ((:lambda (substr)
                        (mu:fix
                         (:lambda (nth)
                            (:if (core:numberp nth)
                                 (:if (mu:eq nth (mu:sv-len str))
                                      ()
                                      (:if (mu:fx-lt nth (mu:fx-add 1 end))
                                           ((:lambda ()
                                               (mu:write (mu:sv-ref str nth) () substr)
                                               (mu:fx-add 1 nth)))
                                           ()))
                                 nth))
                         start)
                        (mu:get-str substr))
                     (mu:open :string :output ""))
                    (core:raise end 'core:substr "end is not a fixnum"))
               (core:raise str 'core:substr "is not a string"))
          (core:raise start 'core:substr "start is not a fixnum"))))

