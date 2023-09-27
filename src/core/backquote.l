;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  backquote
;;;
(mu:intern core "bq-str" (mu:open :string :output ""))

;;;
;;; backquote comma:
;;;
(mu:intern core "%bq-comma"
   (:lambda (stream in-bq-list)
      (core:warn stream "entering bq-comma")                 
      ((:lambda (ch)
          (:if (mu:eq ch #\@)
               (:if in-bq-list
                    (core:read stream () ())
                    (core:error "no @ allowed outside of list" stream))
               (:if (mu:eq ch #\,)
                    (core:%bq-comma stream in-bq-list)
                    ((:lambda ()
                        (core:unread-char ch stream)
                        (:if in-bq-list
                             `(mu:cons ,(core:read stream () ()) ())
                             (core:read stream () ())))))))
                (core:read-char stream () ()))))

;;;
;;; backquote list element:
;;;
;;;     return compilable backquote function call
;;;
(mu:intern core "%bq-list-element"
   (:lambda (expr)
      ((:lambda (string)
          ((:lambda (obj)
              (:if (core:symbolp obj)
                   `(mu:cons ,obj ())
                   `(,obj)))
           (core:%bq-read (mu:open :string :input string) () ())))
       (core:get-string core:%bq-str))))

;;;
;;; backquote list:
;;;
;;;    return compilable backquote function call
;;;

;;; `(core:append (mu:cons ,expr (mu:cons ,(core:%bq-list stream) ())))
(mu:intern core "%bq-list"
   (:lambda (stream in-bq-list)
      ((:lambda (form)
          (:if (mu:eq form core:%read-list-eol%)
               ()
               ((:lambda (append-tail)
                   `(mu:append ,form ,append-tail))
                 `(mu:cons ,(core:%bq-list stream :t) ()))))
       (core:%bq-read stream :t :t))))

;;;
;;; parser
;;;
(mu:intern core "%bq-read-dispatch"
   (:lambda (ch stream in-bq-list)
      ((:lambda (type dispatch-table)
          (:if (mu:eq type :tmacro)
               (:if (core:%andf (core:null in-bq-list) (mu:eq #\( ch))
                    ((:lambda (macro-table)
                        ((:lambda (syntax-pair)
                            (:if (core:null syntax-pair)
                                 (core:%read-macro ch stream)
                                 (core:warn (mu:apply
                                  (mu:sy-val (mu:cdr syntax-pair))
                                  `(,stream ,in-bq-list))
                                  "bq-list")          ))
                         (core:%assoc ch macro-table)))
                     '((#\` . core:%bq-reader)
                       (#\, . core:%bq-comma)
                       (#\( . core:%bq-list)))
                    (core:%read-macro ch stream))
               ((:lambda ()
                    (mu:apply
                     (mu:sy-val (mu:cdr (core:%assoc type dispatch-table)))
                     `(,ch ,stream))))))
       (core:%read-char-syntax ch)
       '((:const   . core:%read-atom)
         (:escape  . core:%read-atom)
         (:macro   . core:%read-macro)
         (:mescape . core:%read-atom)))))

;;;
;;; backquote reader
;;;
(mu:intern core "%bq-read"
   (:lambda (stream in-bq-list recursivep)
      ((:lambda (ch)
          (:if (core:eof stream)
               ()
               ((:lambda (macro)
                   (:if macro
                        (core:apply (mu:car macro) `(,stream ,ch))
                        (core:%bq-read-dispatch ch stream in-bq-list)))
                (core:get-macro-character ch))))
       (core:%read-consume-ws stream))))

(mu:intern core "%bq-reader"
   (:lambda (ch stream)
      (core:%bq-read stream () ())))
