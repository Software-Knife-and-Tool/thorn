;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  backquote
;;;

;;;
;;; backquote comma:
;;;
(mu:intern :prelude "%bq-comma"
   (:lambda (stream in-bq-list-p)
      ((:lambda (char)
          (:if (mu:eq char #\@)
               (:if in-bq-list-p
                    (prelude:read stream () ())
                    (prelude:error "@ not allowed outside of list" stream))
               (:if (mu:eq char #\,)
                    (prelude:%bq-comma stream in-bq-list-p)
                    ((:lambda ()
                        (prelude:unread-char char stream)
                        (:if in-bq-list-p
                             ((:lambda (form)
                                       form)
                                 ;;; `(mu:cons ,form ()))
                              (prelude:read stream () ()))
                             (prelude:read stream () ())))))))
                (prelude:read-char stream () ()))))

;;;
;;; backquote list element:
;;;
;;;     return compilable form
;;;
(mu:intern :prelude "%bq-list-element"
   (:lambda (form)
     `(:quote ,(mu:cons form ()))))

;;;
;;; backquote list:
;;;
;;;    return compilable append form
;;;
(mu:intern :prelude "%bq-list"
   (:lambda (stream in-bq-list-p)
      ((:lambda (form)
          (:if (mu:eq form prelude:%read-list-eol%)
               ()
               ((:lambda (quoted-list append-tail)
                  ((:lambda (args)
                     `(prelude:append ,args))
                   `(,quoted-list ,append-tail)))
                (prelude:%bq-list-element form)
                (prelude:%bq-list stream in-bq-list-p))))
       (prelude:%bq-read stream :t :t))))

;;;
;;; parser
;;;
(mu:intern :prelude "%bq-read-dispatch"
   (:lambda (char stream in-bq-list-p)
      ((:lambda (type dispatch-table)
          (:if (mu:eq type :tmacro)
               (:if (prelude:%andf (prelude:null in-bq-list-p) (mu:eq #\( char))
                    ((:lambda (macro-table)
                        ((:lambda (syntax-pair)
                            (:if (prelude:null syntax-pair)
                                 (prelude:%read-macro char stream)
                                 (mu:apply
                                  (mu:sy-val (mu:cdr syntax-pair))
                                  `(,stream ,in-bq-list-p))))
                         (prelude:%assoc char macro-table)))
                     '((#\` . prelude:%bq-reader)
                       (#\, . prelude:%bq-comma)
                       (#\( . prelude:%bq-list)))
                    (:if (mu:eq char #\,)
                         (prelude:%bq-comma stream in-bq-list-p)
                         (prelude:%read-macro char stream)))
               ((:lambda ()
                    (mu:apply
                     (mu:sy-val (mu:cdr (prelude:%assoc type dispatch-table)))
                     `(,char ,stream))))))
       (prelude:%read-char-syntax char)
       '((:const   . prelude:%read-atom)
         (:escape  . prelude:%read-atom)
         (:macro   . prelude:%read-macro)
         (:mescape . prelude:%read-atom)))))

;;;
;;; backquote reader
;;;
(mu:intern :prelude "%bq-read"
   (:lambda (stream in-bq-list-p recursivep)
      ((:lambda (char)
          (:if (prelude:eof stream)
               ()
               ((:lambda (macro)
                   (:if macro
                        (prelude:apply (mu:car macro) `(,stream ,char))
                        (prelude:%bq-read-dispatch char stream in-bq-list-p)))
                (prelude:get-macro-character char))))
       (prelude:%read-consume-ws stream))))

;;;
;;; external interface
;;;
;;;     the unused char variable is an artifact of the general
;;;     reader dispatch protocol. in this case, we don't need to
;;;     know what macro character got us here, it's always backquote.
;;;
(mu:intern :prelude "%bq-reader"
   (:lambda (char stream)
      (prelude:%bq-read stream () ())))
