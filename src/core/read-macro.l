;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; read macros
;;;

;;;
;;; sharpsign readers
;;;
(mu:intern :core "%read-sharp-char"
   (:lambda (ch stream)
      (core:read-char stream () ())))

(mu:intern :core "%read-sharp-comment"
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
                            (core:raise stream 'core:%read-sharp-comment "unexpected end of file")
                            (core:read-char stream () ())))
                      (core:null loop)))
               (:if (core:eof stream)
                    (core:raise stream 'core:%read-sharp-comment "unexpected end of file")
                    (core:read-char stream () ())))))
       ())))

(mu:intern :core "%read-sharp-vector"
   (:lambda (ch stream)
      ((:lambda (type)
          (:if (core:keywordp type)
               (mu:make-sv type (core:%read-list #\( stream))
               (core:raise type 'core:%read-sharp-vector "not a type keyword")))
          (core:%read stream))))

(mu:intern :core "%read-byte-bits"
   (:lambda (stream)
     (mu:cdr
      (mu:fix
       (:lambda (loop)
         ((:lambda (nbits byte)
            (:if (core:zerop nbits)
                 loop
                 ((:lambda (ch)                   
                    (:if (mu:eof stream)
                         (mu:cons 0 (mu:cons (mu:fx-sub 8 nbits) (mu:fx-ash byte nbits)))
                         ((:lambda (syntax)
                            (:if (core:%orf (mu:eq :tmacro syntax) (mu:eq :wspace syntax))
                                 ((:lambda ()
                                    (:if (mu:eq :tmacro syntax)
                                         (core:unread-char ch stream)
                                         ())
                                    (mu:cons 0 (mu:cons (mu:fx-sub 8 nbits) (mu:fx-ash byte nbits)))))
                                 (:if (core:%orf (mu:eq #\0 ch) (mu:eq #\1 ch))
                                      (mu:cons (core:1- nbits)
                                               (mu:logor
                                                (:if (mu:eq #\1 ch) 1 0)
                                                (mu:fx-mul byte 2)))
                                      (core:raise ch 'core:%read-bit-vector "not a bit designator"))))
                          (core:%read-char-syntax ch))))
                  (core:read-char stream () ()))))
          (mu:car loop)
          (mu:cdr loop)))
        (mu:cons 8 0)))))
             
(mu:intern :core "%read-bit-vector"
   (:lambda (ch stream)
     (mu:fix
       (:lambda (loop)
         (:if (core:vectorp loop)
              loop
              ((:lambda (byte-descriptor)
                 (:if (core:consp byte-descriptor)
                      ((:lambda (length byte)
                         (core:%make-vector
                          (mu:make-sv :byte (core:reverse (mu:cons byte loop)))
                          ()
                          (mu:cons (mu:fx-add length (mu:fx-mul 8 (mu:length loop))) ())))
                       (mu:car byte-descriptor)
                       (mu:cdr byte-descriptor))
                       (mu:cons byte-descriptor loop)))
               (core:%read-byte-bits stream))))
       ())))

(mu:intern :core "%read-sharp-symbol"
   (:lambda (ch stream)
     ((:lambda (symbol)
          (:if (mu:eq :symbol (mu:type-of symbol))
               (mu:make-sy (mu:sy-name symbol))
               (core:raise symbol 'core:%read-sharp-symbol "not a symbol")))
       (core:%read-atom ch stream))))

(mu:intern :core "%read-sharp-number"
  (:lambda (base stream)
    (mu:fix
      (:lambda (loop)
        (:if (core:%orf (core:eof stream) (core:numberp loop))
             loop
             ((:lambda (ch)
                ((:lambda (syntax-type)
                   (:if (mu:eq syntax-type :const)
                        ((:lambda ()
                            (core:write-char ch core:%reader-stream)
                            (core:null loop)))
                        ((:lambda ()
                            (:if (core:null ch)
                                 ()
                                 (core:unread-char ch stream))
                            (:if (mu:eq base #\x)
                                 (core:parse-integer (mu:get-str core:%reader-stream) 16)
                                 (:if (mu:eq base #\b)
                                      (core:parse-integer (mu:get-str core:%reader-stream) 2)
                                      (core:parse-integer (mu:get-str core:%reader-stream) 10)))))))
                 (core:%read-char-syntax ch)))
              (core:read-char stream () ()))))
      ())))

(mu:intern :core "%read-sharp"
  (:lambda (ch stream)
    ((:lambda (ch sharp-table)
       (mu:apply
        (mu:sy-val (mu:cdr (core:%assoc ch sharp-table)))
        (core:%list-2 ch stream)))
     (core:read-char stream () ())
     '((#\| . core:%read-sharp-comment)
       (#\\ . core:%read-sharp-char)
       (#\( . core:%read-sharp-vector)
       (#\* . core:%read-bit-vector)
       (#\b . core:%read-sharp-number)
       (#\x . core:%read-sharp-number)
       (#\d . core:%read-sharp-number)
       (#\: . core:%read-sharp-symbol)))))

;;;
;;; list reader
;;;
(mu:intern :core "%read-list-eol%" (mu:make-sy "eol"))

(mu:intern :core "%read-list-eol"
   (:lambda (ch stream)
      core:%read-list-eol%))

(mu:intern :core "%read-list"
   (:lambda (ch stream)
      (mu:fix
       (:lambda (loop)
          (:if (core:eof stream)
               stream
               ((:lambda (el)
                   (:if (core:%orf (core:eof stream) (mu:eq el core:%read-list-eol%))
                        loop
                        (:if (mu:eq :symbol (mu:type-of el))
                             (:if (mu:eq (mu:sy-name el) ".")
                                  ((:lambda (dotted)
                                      ;;; (core:raise-if (core:null dotted) () 'core:%read-list "early eof in dotted list")
                                      (:if (mu:eq (core:%read stream) core:%read-list-eol%)
                                           ((:lambda ()
                                               (mu:un-char #\) stream)
                                               dotted))
                                           (core:raise () 'core:%read-list "malformed dotted list")))
                                   (core:foldr mu:cons (core:%read stream) loop))
                                  `(,@loop ,el))
                             `(,@loop ,el))))
                (core:%read stream))))
            ())))

;;;
;;; string reader
;;;
(mu:intern :core "%read-string"
   (:lambda (ch stream)
     (mu:fix
      (:lambda (loop)
        (:if (core:stringp loop)
             loop
             ((:lambda (ch)
                (:if (mu:eq ch #\")
                     (core:get-output-stream-string core:%reader-stream)
                     ((:lambda ()
                        (core:write-char ch core:%reader-stream)
                        (core:null loop)))))
              (core:read-char stream () ()))))
        ())))

;;;
;;; quote reader
;;;
(mu:intern :core "%read-quote"
  (:lambda (ch stream)
     (core:%list-2 :quote (core:%read stream))))

;;;
;;; line comment
;;;
(mu:intern :core "%read-line-comment"
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
(mu:intern :core "%read-macro"
  (:lambda (ch stream)
     ((:lambda (macro-table)
       (mu:apply
            (mu:sy-val (mu:cdr (core:%assoc ch macro-table)))
            (core:%list-2 ch stream)))
       '((#\" . core:%read-string)
         (#\# . core:%read-sharp)
         (#\' . core:%read-quote)
         (#\` . core:%bq-reader)
         (#\( . core:%read-list)
         (#\) . core:%read-list-eol)
         (#\; . core:%read-line-comment)))))

;;;
;;; get/set-macro-character
;;;
(mu:intern :core "get-macro-character"
   (:lambda (char)
     (:if (core:charp char)
          ((:lambda (macro-symbol)
              (:if macro-symbol
                   (core:symbol-value macro-symbol)
                   ()))
           (mu:ns-find core:%char-macro-ns% (mu:make-sv :char (mu:cons char ()))))
          (core:raise char 'core:get-macro-character "not a character"))))

(mu:intern :core "set-macro-character"
   (:lambda (char fn non-terminating)
      (:if (core:charp char)
         (:if (core:functionp fn)
              (mu:intern core:%char-macro-ns% (mu:make-sv :char (mu:cons char ())) (mu:cons fn non-terminating))
              (core:raise fn 'core:set-macro-character "not a function"))
         (core:raise char 'core:set-macro-character "not a character"))
      :t))
