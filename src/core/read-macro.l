;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; read macros
;;;

;;;
;;; sharpsign readers
;;;
(mu:intern core :intern "read-sharp-char"
   (:lambda (ch stream)
      (core:read-char stream () ())))

(mu:intern core :intern "read-sharp-comment"
    (:lambda (ch stream)
      (mu:fix
       (:lambda (loop)
         (:if (core:streamp loop)
              loop
              ((:lambda (ch)
                 (:if (mu:eq ch #\|)
                      ((:lambda (ch)
                         (:if (mu:eq ch #\#)
                              stream
                              (core:null loop)))
                       (:if (core:eof stream)
                            (core:raise stream 'core::read-sharp-comment "unexpected end of file")
                            (core:read-char stream () ())))
                      (core:null loop)))
               (:if (core:eof stream)
                    (core:raise stream 'core::read-sharp-comment "unexpected end of file")
                    (core:read-char stream () ())))))
       ())))

(mu:intern core :intern "read-sharp-vector"
    (:lambda (ch stream)
      ((:lambda (type)
          (core:raise-unless core:keywordp type 'core::read-sharp-vector "not a type keyword")
          (mu:make-sv type (core::read-list #\( stream)))
       (core::read-atom ch stream))))

(mu:intern core :intern "read-sharp-symbol"
   (:lambda (ch stream)
     ((:lambda (symbol)
          (:if (mu:eq :symbol (mu:type-of symbol))
               (mu:make-sy (mu:sy-name symbol))
               (core:raise symbol 'core::read-sharp-symbol "not a symbol")))
       (core::read-atom ch stream))))

(mu:intern core :intern "read-sharp-number"
  (:lambda (base stream)
    (mu:fix
      (:lambda (loop)
        (:if (core::orf (core:eof stream) (core:numberp loop))
             loop
             ((:lambda (ch)
                ((:lambda (syntax-type)
                   (:if (mu:eq syntax-type :const)
                        (core::prog2
                            (core:write-char ch core::reader-stream)
                            (core:null loop))
                        (core::prog2
                            (:if (core:null ch)
                                 ()
                                 (core:unread-char ch stream))
                            (:if (mu:eq base #\x)
                                 (core:parse-integer (mu:get-str core::reader-stream) 16)
                                 (:if (mu:eq base #\b)
                                      (core:parse-integer (mu:get-str core::reader-stream) 2)
                                      (core:parse-integer (mu:get-str core::reader-stream) 10))))))
                 (core::read-char-syntax ch)))
              (core:read-char stream () ()))))
      ())))

(mu:intern core :intern "read-sharp"
  (:lambda (ch stream)
    ((:lambda (ch sharp-table)
       (mu:apply
        (mu:sy-val (mu:cdr (core:assoc ch sharp-table)))
        `(,ch ,stream)))
     (core:read-char stream () ())
     '((#\| . core::read-sharp-comment)
       (#\\ . core::read-sharp-char)
       (#\( . core::read-sharp-vector)
       (#\b . core::read-sharp-number)
       (#\x . core::read-sharp-number)
       (#\d . core::read-sharp-number)
       (#\: . core::read-sharp-symbol)))))

;;;
;;; list reader
;;;
(mu:intern core :intern "read-list-eol" (mu:make-sy "eol"))
(mu:intern core :intern "read-list"
  (:lambda (ch stream)
    (:if (mu:eq ch #\))
         core::read-list-eol
         (mu:fix
           (:lambda (loop)
             (:if (core:eof stream)
                  stream
                  ((:lambda (el)
                     (:if (core::orf (core:eof stream) (mu:eq el core::read-list-eol))
                          loop
                          (:if (mu:eq :symbol (mu:type-of el))
                               (:if (mu:eq (mu:sy-name el) ".")
                                    ((:lambda (dotted)
                                       (core:raise-if (core:null dotted) () 'core::read-list "early eof in dotted list")
                                       (:if (mu:eq (core::read stream) core::read-list-eol)
                                            (core::prog2
                                                (mu:un-char #\) stream)
                                                dotted)
                                            (core:raise () 'core::read-list "malformed dotted list")))
                                     (core:foldr mu:cons (core::read stream) loop))
                                    `(,@loop ,el))
                               `(,@loop ,el))))
                   (core::read stream))))
          ()))))

;;;
;;; string reader
;;;
(mu:intern core :intern "read-string"
   (:lambda (ch stream)
     (mu:fix
      (:lambda (loop)
        (:if (core:stringp loop)
             loop
             ((:lambda (ch)
                (:if (mu:eq ch #\")
                     (core:get-output-stream-string core::reader-stream)
                     (core::prog2
                        (core:write-char ch core::reader-stream)
                        (core:null loop))))
              (core:read-char stream () ()))))
        ())))

;;;
;;; quote reader
;;;
(mu:intern core :intern "read-quote"
  (:lambda (ch stream)
     `(:quote ,(core::read stream))))

;;;
;;; line comment
;;;
(mu:intern core :intern "read-line-comment"
    (:lambda (ch stream)
      (mu:fix
       (:lambda (loop)
         (:if (core:streamp loop)
              loop
              ((:lambda (ch)
                 (:if (mu:eq ch #\linefeed)
                      stream
                      (core:null loop)))
               (core:read-char stream () ()))))
       ())))

;;;
;;; read macros
;;;
(mu:intern core :intern "read-macro"
  (:lambda (ch stream)
    ((:lambda (macro-table)
       (mu:apply
            (mu:sy-val (mu:cdr (core:assoc ch macro-table)))
            `(,ch ,stream)))
       '((#\" . core::read-string)
         (#\# . core::read-sharp)
         (#\' . core::read-quote)
         (#\( . core::read-list)
         (#\) . core::read-list)
         (#\; . core::read-line-comment)))))

;;;
;;; get/set-macro-character
;;;
(mu:intern core :extern "get-macro-character"
   (:lambda (char)
     (core:raise-unless core:charp char 'core:get-macro-character "not a character")
     ((:lambda (macro-symbol)
        (:if macro-symbol
            (core:symbol-value macro-symbol)
            ()))
      (mu:ns-find core::core-char-macro-ns :extern (mu:make-sv :char `(,char))))))

(mu:intern core :extern "set-macro-character"
   (:lambda (char fn non-terminating)
     (core:raise-unless core:charp char 'core:set-macro-character "not a character")
     (core:raise-unless core:functionp fn 'core:set-macro-character "not a function")
     (mu:intern core::core-char-macro-ns :extern (mu:make-sv :char `(,char)) (mu:cons fn non-terminating))
     :t))
