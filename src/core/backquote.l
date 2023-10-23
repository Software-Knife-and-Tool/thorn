;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  backquote
;;;

;;;
;;; backquote comma:
;;;
(mu:intern :core "%bq-comma"
   (:lambda (stream in-bq-list-p)
      ((:lambda (char)
          (:if (mu:eq char #\@)
               (:if in-bq-list-p
                    (core:read stream () ())
                    (core:error "@ not allowed outside of list" stream))
               (:if (mu:eq char #\,)
                    (core:%bq-comma stream in-bq-list-p)
                    ((:lambda ()
                        (core:unread-char char stream)
                        (:if in-bq-list-p
                             ((:lambda (form)
                                       form)
                                 ;;; `(mu:cons ,form ()))
                              (core:read stream () ()))
                             (core:read stream () ())))))))
                (core:read-char stream () ()))))

;;;
;;; backquote list element:
;;;
;;;     return compilable form
;;;
(mu:intern :core "%bq-list-element"
   (:lambda (form)
     `(:quote ,(mu:cons form ()))))

;;;
;;; backquote list:
;;;
;;;    return compilable append form
;;;
(mu:intern :core "%bq-list"
   (:lambda (stream in-bq-list-p)
      ((:lambda (form)
          (:if (mu:eq form core:%read-list-eol%)
               ()
               ((:lambda (quoted-list append-tail)
                   `(mu:%append ,quoted-list ,append-tail))
                (core:%bq-list-element form)
                (core:%bq-list stream in-bq-list-p))))
       (core:%bq-read stream :t :t))))

;;;
;;; parser
;;;
(mu:intern :core "%bq-read-dispatch"
   (:lambda (char stream in-bq-list-p)
      ((:lambda (type dispatch-table)
          (:if (mu:eq type :tmacro)
               (:if (core:%andf (core:null in-bq-list-p) (mu:eq #\( char))
                    ((:lambda (macro-table)
                        ((:lambda (syntax-pair)
                            (:if (core:null syntax-pair)
                                 (core:%read-macro char stream)
                                 (mu:apply
                                  (mu:sy-val (mu:cdr syntax-pair))
                                  `(,stream ,in-bq-list-p))))
                         (core:%assoc char macro-table)))
                     '((#\` . core:%bq-reader)
                       (#\, . core:%bq-comma)
                       (#\( . core:%bq-list)))
                    (:if (mu:eq char #\,)
                         (core:%bq-comma stream in-bq-list-p)
                         (core:%read-macro char stream)))
               ((:lambda ()
                    (mu:apply
                     (mu:sy-val (mu:cdr (core:%assoc type dispatch-table)))
                     `(,char ,stream))))))
       (core:%read-char-syntax char)
       '((:const   . core:%read-atom)
         (:escape  . core:%read-atom)
         (:macro   . core:%read-macro)
         (:mescape . core:%read-atom)))))

;;;
;;; backquote reader
;;;
(mu:intern :core "%bq-read"
   (:lambda (stream in-bq-list-p recursivep)
      ((:lambda (char)
          (:if (core:eof stream)
               ()
               ((:lambda (macro)
                   (:if macro
                        (core:apply (mu:car macro) `(,stream ,char))
                        (core:%bq-read-dispatch char stream in-bq-list-p)))
                (core:get-macro-character char))))
       (core:%read-consume-ws stream))))

;;;
;;; external interface
;;;
;;;     the unused char variable is an artifact of the general
;;;     reader dispatch protocol. in this case, we don't need to
;;;     know what macro character got us here, it's always backquote.
;;;
(mu:intern :core "%bq-reader"
   (:lambda (char stream)
      (core:%bq-read stream () ())))
