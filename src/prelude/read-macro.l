;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; read macros
;;;

;;;
;;; sharpsign readers
;;;
(mu:intern :prelude "%read-sharp-char"
   (:lambda (ch stream)
      (prelude:read-char stream () ())))

(mu:intern :prelude "%read-sharp-comment"
    (:lambda (ch stream)
      (mu:fix
       (:lambda (loop)
         (:if (prelude:streamp loop)
              loop
              ((:lambda (ch)
                 (:if (mu:eq ch #\|)
                      ((:lambda (ch)
                         (:if (mu:eq ch #\#)
                              stream
                              (prelude:null loop)))
                       (:if (prelude:eof stream)
                            (prelude:raise stream 'prelude:%read-sharp-comment "unexpected end of file")
                            (prelude:read-char stream () ())))
                      (prelude:null loop)))
               (:if (prelude:eof stream)
                    (prelude:raise stream 'prelude:%read-sharp-comment "unexpected end of file")
                    (prelude:read-char stream () ())))))
       ())))

(mu:intern :prelude "%read-sharp-vector"
   (:lambda (ch stream)
      ((:lambda (type)
          (:if (prelude:keywordp type)
               (mu:make-sv type (prelude:%read-list #\( stream))
               (prelude:raise type 'prelude:%read-sharp-vector "not a type keyword")))
          (prelude:%read stream))))

(mu:intern :prelude "%read-byte-bits"
   (:lambda (stream)
     (mu:cdr
      (mu:fix
       (:lambda (loop)
         ((:lambda (nbits byte)
            (:if (prelude:zerop nbits)
                 loop
                 ((:lambda (ch)                   
                    (:if (mu:eof stream)
                         (mu:cons 0 (mu:cons (mu:fx-sub 8 nbits) (mu:fx-ash byte nbits)))
                         ((:lambda (syntax)
                            (:if (prelude:%orf (mu:eq :tmacro syntax) (mu:eq :wspace syntax))
                                 ((:lambda ()
                                    (:if (mu:eq :tmacro syntax)
                                         (prelude:unread-char ch stream)
                                         ())
                                    (mu:cons 0 (mu:cons (mu:fx-sub 8 nbits) (mu:fx-ash byte nbits)))))
                                 (:if (prelude:%orf (mu:eq #\0 ch) (mu:eq #\1 ch))
                                      (mu:cons (prelude:1- nbits)
                                               (mu:logor
                                                (:if (mu:eq #\1 ch) 1 0)
                                                (mu:fx-mul byte 2)))
                                      (prelude:raise ch 'prelude:%read-sharp-bit-vector "not a bit designator"))))
                          (prelude:%read-char-syntax ch))))
                  (prelude:read-char stream () ()))))
          (mu:car loop)
          (mu:cdr loop)))
        (mu:cons 8 0)))))
             
(mu:intern :prelude "%read-sharp-bit-vector"
   (:lambda (ch stream)
     (mu:fix
       (:lambda (loop)
         (:if (prelude:vectorp loop)
              loop
              ((:lambda (byte-descriptor)
                 (:if (prelude:consp byte-descriptor)
                      ((:lambda (length byte)
                         (prelude:%make-vector
                          (mu:make-sv :byte (prelude:reverse (mu:cons byte loop)))
                          ()
                          (mu:cons (mu:fx-add length (mu:fx-mul 8 (mu:length loop))) ())))
                       (mu:car byte-descriptor)
                       (mu:cdr byte-descriptor))
                       (mu:cons byte-descriptor loop)))
               (prelude:%read-byte-bits stream))))
       ())))

(mu:intern :prelude "%read-sharp-dot"
   (:lambda (ch stream)
      (mu:eval (prelude:compile (prelude:read stream () ())))))

(mu:intern :prelude "%read-sharp-symbol"
   (:lambda (ch stream)
     ((:lambda (symbol)
          (:if (mu:eq :symbol (mu:type-of symbol))
               (mu:make-sy (mu:sy-name symbol))
               (prelude:raise symbol 'prelude:%read-sharp-symbol "not a symbol")))
       (prelude:%read-atom ch stream))))

(mu:intern :prelude "%read-sharp-number"
  (:lambda (base stream)
    (mu:fix
      (:lambda (loop)
        (:if (prelude:%orf (prelude:eof stream) (prelude:numberp loop))
             loop
             ((:lambda (ch)
                ((:lambda (syntax-type)
                   (:if (mu:eq syntax-type :const)
                        ((:lambda ()
                            (prelude:write-char ch prelude:%reader-stream)
                            (prelude:null loop)))
                        ((:lambda ()
                            (:if (prelude:null ch)
                                 ()
                                 (prelude:unread-char ch stream))
                            (:if (mu:eq base #\x)
                                 (prelude:parse-integer (mu:get-str prelude:%reader-stream) 16)
                                 (:if (mu:eq base #\b)
                                      (prelude:parse-integer (mu:get-str prelude:%reader-stream) 2)
                                      (prelude:parse-integer (mu:get-str prelude:%reader-stream) 10)))))))
                 (prelude:%read-char-syntax ch)))
              (prelude:read-char stream () ()))))
      ())))

(mu:intern :prelude "%read-sharp"
  (:lambda (ch stream)
    ((:lambda (ch sharp-table)
       (mu:apply
        (mu:sy-val (mu:cdr (prelude:%assoc ch sharp-table)))
        (prelude:%list-2 ch stream)))
     (prelude:read-char stream () ())
     '((#\| . prelude:%read-sharp-comment)
       (#\( . prelude:%read-sharp-vector)
       (#\* . prelude:%read-sharp-bit-vector)
       (#\. . prelude:%read-sharp-dot)
       (#\\ . prelude:%read-sharp-char)
       (#\b . prelude:%read-sharp-number)
       (#\x . prelude:%read-sharp-number)
       (#\d . prelude:%read-sharp-number)
       (#\: . prelude:%read-sharp-symbol)))))

;;;
;;; list reader
;;;
(mu:intern :prelude "%read-list-eol%" (mu:make-sy "eol"))

(mu:intern :prelude "%read-list-eol"
   (:lambda (ch stream)
      prelude:%read-list-eol%))

(mu:intern :prelude "%read-list"
   (:lambda (ch stream)
      (mu:fix
       (:lambda (loop)
          (:if (prelude:eof stream)
               stream
               ((:lambda (el)
                   (:if (prelude:%orf (prelude:eof stream) (mu:eq el prelude:%read-list-eol%))
                        loop
                        (:if (mu:eq :symbol (mu:type-of el))
                             (:if (mu:eq (mu:sy-name el) ".")
                                  ((:lambda (dotted)
                                      ;;; (prelude:raise-if (prelude:null dotted) () 'prelude:%read-list "early eof in dotted list")
                                      (:if (mu:eq (prelude:%read stream) prelude:%read-list-eol%)
                                           ((:lambda ()
                                               (mu:un-char #\) stream)
                                               dotted))
                                           (prelude:raise () 'prelude:%read-list "malformed dotted list")))
                                   (prelude:foldr mu:cons (prelude:%read stream) loop))
                                  `(,@loop ,el))
                             `(,@loop ,el))))
                (prelude:%read stream))))
            ())))

;;;
;;; string reader
;;;
(mu:intern :prelude "%read-string"
   (:lambda (ch stream)
     (mu:fix
      (:lambda (loop)
        (:if (prelude:stringp loop)
             loop
             ((:lambda (ch)
                (:if (mu:eq ch #\")
                     (prelude:get-output-stream-string prelude:%reader-stream)
                     ((:lambda ()
                        (prelude:write-char ch prelude:%reader-stream)
                        (prelude:null loop)))))
              (prelude:read-char stream () ()))))
        ())))

;;;
;;; quote reader
;;;
(mu:intern :prelude "%read-quote"
  (:lambda (ch stream)
     (prelude:%list-2 :quote (prelude:%read stream))))

;;;
;;; line comment
;;;
(mu:intern :prelude "%read-line-comment"
    (:lambda (ch stream)
      (mu:fix
       (:lambda (loop)
         (:if (prelude:streamp loop)
              loop
              ((:lambda (ch)
                 (:if (mu:eq ch #\linefeed)
                      stream
                      (prelude:null loop)))
               (prelude:read-char stream () ()))))
       ())))

;;;
;;; read macros
;;;
(mu:intern :prelude "%read-macro"
  (:lambda (ch stream)
     ((:lambda (macro-table)
       (mu:apply
            (mu:sy-val (mu:cdr (prelude:%assoc ch macro-table)))
            (prelude:%list-2 ch stream)))
       '((#\" . prelude:%read-string)
         (#\# . prelude:%read-sharp)
         (#\' . prelude:%read-quote)
         (#\` . prelude:%bq-reader)
         (#\( . prelude:%read-list)
         (#\) . prelude:%read-list-eol)
         (#\; . prelude:%read-line-comment)))))

;;;
;;; get/set-macro-character
;;;
(mu:intern :prelude "get-macro-character"
   (:lambda (char)
     (:if (prelude:charp char)
          ((:lambda (macro-symbol)
              (:if macro-symbol
                   (prelude:symbol-value macro-symbol)
                   ()))
           (mu:ns-find prelude:%char-macro-ns% (mu:make-sv :char (mu:cons char ()))))
          (prelude:raise char 'prelude:get-macro-character "not a character"))))

(mu:intern :prelude "set-macro-character"
   (:lambda (char fn non-terminating)
      (:if (prelude:charp char)
         (:if (prelude:functionp fn)
              (mu:intern prelude:%char-macro-ns% (mu:make-sv :char (mu:cons char ())) (mu:cons fn non-terminating))
              (prelude:raise fn 'prelude:set-macro-character "not a function"))
         (prelude:raise char 'prelude:set-macro-character "not a character"))
      :t))
