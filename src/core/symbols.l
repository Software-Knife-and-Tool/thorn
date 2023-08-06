;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  symbols
;;;
(mu:intern core :extern "boundp" mu:boundp)
(mu:intern core :extern "fboundp"
  (:lambda (symbol)
    (:if (core:boundp symbol)
         (core::appliablep (mu:sy-val symbol))
         ())))

(mu:intern core :extern "uninternedp"
   (:lambda (sym)
      (:if (core:null sym)
           ()
           (core:null (mu:sy-ns sym)))))

(mu:intern core :extern "keywordp"
   (:lambda (t)
      (mu:eq (core:type-of t) 'keyword)))

(mu:intern core :extern "symbolp"
   (:lambda (t)
      (core::orf (core::orf (core:keywordp t) (mu:eq () t))
                (mu:eq 'symbol (core:type-of t)))))

;;;
;;; utilities
;;;
(mu:intern core :extern "symbol-name" mu:sy-name)
(mu:intern core :extern "symbol-ns" mu:sy-ns)
(mu:intern core :extern "symbol-value" mu:sy-val)

(mu:intern core :extern "keyword"
   (:lambda (name)
      (core:raise-unless core:stringp name 'core:keyword "not a keyword name")
      (:if (mu:fx-lt (mu:sv-len name) 8)
           (mu:keyword name)
           (core::make-ctype "keyword" `(,(mu:cons :name name))))))

(mu:intern core :intern "intern"
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
