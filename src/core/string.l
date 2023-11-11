;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; strings
;;;
(mu:intern :core "stringp"
   (:lambda (string)
      (:if (mu:eq (mu:type-of string) :vector)
           (mu:eq :char (mu:sv-type string))
           (:if (core:core-type-p string)
                (:if (mu:eq "vector" (mu:sv-ref (mu:st-vec string) 0))
                     (mu:eq :char (mu:sv-type (core:%vector-prop :base string)))
                     ())
                ()))))

(mu:intern :core "get-output-stream-string"
   (:lambda (stream)
      (mu:get-str stream)))

(mu:intern :core "schar"
  (:lambda (str index)
     (:if (core:stringp str)
          (:if (core:fixnump index)
               (mu:sv-ref str index)
               (core:raise index 'core:schar "not a string"))
          (core:raise str 'core:schar "not a string"))))

(mu:intern :core "string="
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

(mu:intern :core "%string-write"
   (:lambda (string escape stream)
      (:if (mu:eq :vector (mu:type-of string))
           (mu:write string escape stream)
           ((:lambda (length)
               (:if escape
                    (mu:write #\" () stream)
                    ())
               (mu:fix
                (:lambda (index)
                   (:if (mu:fx-lt index length)
                        ((:lambda ()
                            (core:write (core:vector-ref string index) () stream)
                            (core:1+ index)))
                        index))
                0)
               (:if escape
                    (mu:write #\" () stream)
                    ()))
            (core:%vector-prop :length string)))
      ()))

;;;
;;; string-position
;;;
(mu:intern :core "string-position"
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
(mu:intern :core "string-find"
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
(mu:intern :core "string"
   (:lambda (designator)
      (:if (core:stringp designator)
         designator
         (:if (mu:eq :symbol (mu:type-of designator))
              (mu:sy-name designator)
              (:if (core:charp designator)
                   (mu:make-sv :char `(,designator))
                   (core:raise designator 'core:string "not a string designator"))))))

(mu:intern :core "string-append"
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

(mu:intern :core "substr"
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

#|
(mu:intern :core "substr-"
  (:lambda (str start end)
     (:if (core:stringp str)
          (:if (core:fixnump start)
               (:if (core:fixnump end)
                    (core:slice str start end)
                    (core:raise end 'core:substr "is not a fixnum"))
               (core:raise str 'core:substr "is not a string"))
          (core:raise start 'core:substr "is not a fixnum"))))
|#

;;;
;;; read line
;;;
(mu:intern :core "read-line"
   (:lambda (stream eof-error eof-value)
      (:if (core:streamp stream)
           ((:lambda (sstream)
               (mu:fix
                (:lambda (loop)
                   (:if (core:stringp loop)
                        loop
                        ((:lambda (ch)
                            (:if (mu:eq ch #\linefeed)
                                 (mu:get-str sstream)
                                 ((:lambda ()
                                     (mu:wr-char ch sstream)
                                     (core:null loop)))))
                         (mu:rd-char stream eof-error eof-value))))
                ()))
            (mu:open :string :output ""))
           (core:raise str 'core:read-line "is not a stream"))))

;;;
;;; read from string
;;;
(mu:intern :core "read-string"
   (:lambda (str eof-error eof-value)
      (:if (core:stringp str)
           (mu:read (mu:open :string :input str) eof-error eof-value)
           (core:raise str 'core:read-string "is not a string"))))

;;;
;;; write string
;;;
(mu:intern :core "write-string"
   (:lambda (str stream)
      (:if (core:stringp str)
           (:if (core:streamp stream)
                (mu:write str () stream)
                (core:raise str 'core:write-string "is not a stream"))
           (core:raise str 'core:write-string "is not a string"))))

;;;
;;; write line
;;;
(mu:intern :core "write-line"
   (:lambda (str stream)
      (core:write-string str stream)
      (core:terpri stream)))
