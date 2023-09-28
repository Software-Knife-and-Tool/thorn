;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  backquote
;;;

;;;
;;; backquote comma:
;;;
(mu:intern core "%bq-comma"
   (:lambda (stream in-bq-list)
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
                             ((:lambda (form) form)
                              (core:read stream () ()))
                             (core:read stream () ())))))))
                (core:read-char stream () ()))))

;;;
;;; backquote list element:
;;;
;;;     return compilable backquote function call
;;;
(mu:intern core "%bq-list-element"
   (:lambda (obj)
      ;;; (core:warn `(,obj ,(mu:type-of obj)) "bq-list-element") 
      (:if (core:symbolp obj)
           ((:lambda (list)
               `(:quote ,list))
            (mu:cons obj ()))
           ((:lambda (list)
               `(:quote ,list))
            (mu:cons obj ())))))

;;;
;;; backquote list:
;;;
;;;    return compilable backquote function call
;;;
(mu:intern core "%bq-list"
   (:lambda (stream in-bq-list)
      ((:lambda (form)
          (:if (mu:eq form core:%read-list-eol%)
               ()
               ((:lambda (quoted-list append-tail)
                   `(mu:%append ,quoted-list ,append-tail))
                (core:%bq-list-element form)
                ((:lambda (el)
                    (:if (core:null el)
                         ()
                         el))
                 (core:%bq-list stream in-bq-list)))))
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
                                 (mu:apply
                                  (mu:sy-val (mu:cdr syntax-pair))
                                  `(,stream ,in-bq-list))))
                         (core:%assoc ch macro-table)))
                     '((#\` . core:%bq-reader)
                       (#\, . core:%bq-comma)
                       (#\( . core:%bq-list)))
                    (:if (mu:eq ch #\,)
                         (core:%bq-comma stream in-bq-list) 
                         (core:%read-macro ch stream)))
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
