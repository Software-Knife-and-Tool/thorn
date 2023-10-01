;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; core reader
;;;
(mu:intern :core "%reader-stream" (mu:open :string :output ""))
(mu:intern :core "%reader-ns-stream" (mu:open :string :output "core"))

;;;
;;; readtable
;;;
(mu:intern :core "%read-char-syntax"
   (:lambda (ch)
     ((:lambda (read-table) (mu:cdr (core:%assoc ch read-table)))
      '((#\return . :wspace) (#\linefeed . :wspace) (#\page . :wspace)
        (#\tab . :wspace) (#\space . :wspace)
        (#\0 . :const)  (#\1 . :const)  (#\2 . :const)  (#\3 . :const)
        (#\4 . :const)  (#\5 . :const)  (#\6 . :const)  (#\7 . :const)
        (#\8 . :const)  (#\9 . :const)  (#\: . :const)  (#\< . :const)
        (#\> . :const)  (#\= . :const)  (#\? . :const)  (#\! . :const)
        (#\@ . :const)  (#\; . :tmacro) (#\" . :tmacro) (#\# . :macro)
        (#\' . :tmacro) (#\( . :tmacro) (#\) . :tmacro) (#\` . :tmacro)
        (#\, . :tmacro) (#\\ . :escape) (#\| . :mescape) (#\a . :const)
        (#\b . :const)  (#\c . :const)  (#\d . :const)  (#\e . :const)
        (#\f . :const)  (#\g . :const)  (#\h . :const)  (#\i . :const)
        (#\j . :const)  (#\k . :const)  (#\l . :const)  (#\m . :const)
        (#\n . :const)  (#\o . :const)  (#\p . :const)  (#\q . :const)
        (#\r . :const)  (#\s . :const)  (#\t . :const)  (#\v . :const)
        (#\w . :const)  (#\x . :const)  (#\y . :const)  (#\z . :const)
        (#\[ . :const)  (#\] . :const)  (#\$ . :const)  (#\* . :const)
        (#\{ . :const)  (#\} . :const)  (#\+ . :const)  (#\- . :const)
        (#\/ . :const)  (#\~ . :const)  (#\. . :const)  (#\% . :const)
        (#\& . :const)  (#\^ . :const)  (#\_ . :const)  (#\a . :const)
        (#\b . :const)  (#\c . :const)  (#\d . :const)  (#\e . :const)
        (#\f . :const)  (#\g . :const)  (#\h . :const)  (#\i . :const)
        (#\j . :const)  (#\k . :const)  (#\l . :const)  (#\m . :const)
        (#\n . :const)  (#\o . :const)  (#\p . :const)  (#\q . :const)
        (#\r . :const)  (#\s . :const)  (#\t . :const)  (#\u . :const)
        (#\v . :const)  (#\w . :const)  (#\x . :const)  (#\y . :const)
        (#\z . :const)))))

(mu:intern :core "%read-number"
  (:lambda (atom)
    ((:lambda (fx)
        (:if fx
             fx
             ((:lambda (fl)
                (:if fl
                     fl
                     ()))
             (core:parse-float atom))))
      (core:parse-integer atom 10))))

;;;
;;; atom reader
;;;
(mu:intern :core "%read-atom"
  (:lambda (ch stream)
    (core:write-char ch core:%reader-stream)
    (mu:fix
     (:lambda (loop)
       (:if (core:consp loop)
            loop
            (:if (core:eof stream)
                 loop
                 ((:lambda (ch)
                    (:if (core:null ch)
                         loop
                         ((:lambda (syntax-type)
                            (:if (mu:eq :const syntax-type)
                                 ((:lambda ()
                                     (core:write-char ch core:%reader-stream)
                                     (core:null loop)))
                                 ((:lambda ()
                                     (core:unread-char ch stream)
                                     loop))))
                          (core:%read-char-syntax ch))))
                  (core:read-char stream () ())))))
     ())
    ((:lambda (token)
       ((:lambda (number)
          (:if number
               number
               (core:%symbol-macro-expand (core:%read-symbol token))))
        (core:%read-number token)))
     (core:get-output-stream-string core:%reader-stream))))

;;;
;;; parser
;;;
(mu:intern :core "%read-dispatch"
  (:lambda (ch stream)
    ((:lambda (dispatch-table)
       (mu:apply
        (mu:sy-val (mu:cdr (core:%assoc (core:%read-char-syntax ch) dispatch-table)))
        `(,ch ,stream)))
     '((:const   . core:%read-atom)
       (:escape  . core:%read-atom)
       (:macro   . core:%read-macro)
       (:tmacro  . core:%read-macro)
       (:mescape . core:%read-atom)))))

;;;
;;; consume whitespace and comments
;;;
(mu:intern :core "%read-consume-ws"
  (:lambda (stream)
    (mu:fix
     (:lambda (loop)
       (:if (core:%orf (core:streamp loop) (core:charp loop))
            loop
            ((:lambda (ch)
               (:if (core:null ch)
                    stream
                    (:if (mu:eq :wspace (core:%read-char-syntax ch))
                         (core:null loop)
                         (:if (mu:eq ch #\#)
                              ((:lambda (ch)
                                 (:if (mu:eq ch #\|)
                                      ((:lambda ()
                                          (core:%read-sharp-comment ch stream)
                                          (core:null loop)))
                                      ((:lambda ()
                                          (core:unread-char ch stream)
                                          #\#))))
                               (core:read-char stream () ()))
                              (:if (mu:eq ch #\;)
                                   ((:lambda ()
                                       (core:%read-line-comment ch stream)
                                       (core:null loop)))
                                   ch)))))
             (core:read-char stream () ()))))
     ())))

;;;
;;; recursive reader
;;;
(mu:intern :core "%read"
  (:lambda (stream)
    ((:lambda (ch)
       (:if (core:eof stream)
            ()
            ((:lambda (macro)
               (:if macro
                    (core:apply (mu:car macro) `(,stream ,ch))
                    (core:%read-dispatch ch stream)))
             (core:get-macro-character ch))))
     (core:%read-consume-ws stream))))
