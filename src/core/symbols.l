;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  symbols
;;;
(mu:intern core::ns :extern "boundp" mu:boundp)
(mu:intern core::ns :extern "fboundp"
  (:lambda (symbol)
    (:if (core:boundp symbol)
         (core::appliablep (mu:sy-val symbol))
         ())))

(mu:intern core::ns :extern "uninternedp"
  (:lambda (sym)
    (core:null (mu:sy-ns sym))))

(mu:intern core::ns :extern "keywordp"
   (:lambda (t)
      (mu:eq (core:type-of t) 'keyword)))

(mu:intern core::ns :extern "symbolp"
   (:lambda (t)
      (core::or (core::or (core:keywordp t) (mu:eq () t))
                (mu:eq 'symbol (core:type-of t)))))

;;;
;;; utilities
;;;
(mu:intern core::ns :extern "symbol-name" mu:sy-name)
(mu:intern core::ns :extern "symbol-ns" mu:sy-ns)
(mu:intern core::ns :extern "symbol-value" mu:sy-val)

(mu:intern core::ns :extern "keyword"
   (:lambda (name)
      (core:raise-unless core:stringp name 'core:keyword "not a keyword name")
      (:if (mu:fx-lt (mu:sv-len name) 8)
           (mu:keyword name)
           (core::make-ctype "keyword" `(,(mu:cons :name name))))))

(mu:intern core::ns :intern "intern"
   (:lambda (symbol value)
      (:if (core:uninternedp symbol)
           ((:lambda (sym)
               (:if sym
                    (:if (core:boundp sym)
                         symbol
                         (mu:intern (core::read-get-ns) :extern (mu:sy-name symbol) value))
                    (mu:intern (core::read-get-ns) :extern (mu:sy-name symbol) value)))
            (mu:ns-find (core::read-get-ns) :extern (mu:sy-name symbol)))
           ((:lambda (ns name)
               (mu:intern
                (core::read-get-ns)
                (:if (mu:ns-find ns :extern name)
                     :extern
                     :intern)
                name
                value))
            (mu:sy-ns symbol)
            (mu:sy-name symbol)))))
