;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  symbols
;;;
(mu:intern core "boundp" mu:boundp)
(mu:intern core "fboundp"
  (:lambda (symbol)
    (:if (core:boundp symbol)
         (core:%appliablep (mu:sy-val symbol))
         ())))

(mu:intern core "uninternedp"
   (:lambda (sym)
      (core:null (core:symbol-ns sym))))

(mu:intern core "keywordp"
   (:lambda (t)
      (mu:eq (core:type-of t) 'keyword)))

(mu:intern core "symbolp"
   (:lambda (t)
      (core:%orf (core:%orf (core:keywordp t) (mu:eq () t))
                (mu:eq 'symbol (core:type-of t)))))

;;;
;;; utilities
;;;
(mu:intern core "symbol-name" mu:sy-name)

(mu:intern core "symbol-ns"
   (:lambda (sym)
      (:if (core:null sym)
           (mu:map-ns "mu")     
           (mu:sy-ns sym))))

(mu:intern core "symbol-value" mu:sy-val)

(mu:intern core "keyword"
   (:lambda (name)
      (core:raise-unless core:stringp name 'core:keyword "not a keyword name")
      (:if (mu:fx-lt (mu:sv-len name) 8)
           (mu:keyword name)
           (core:%make-core-type "keyword" `(,(mu:cons :name name))))))

(mu:intern core "intern"
   (:lambda (symbol value)
      (:if (core:uninternedp symbol)
           ((:lambda (sym)
               (:if sym
                    (:if (core:boundp sym)
                         symbol
                         (mu:intern (core:%read-get-ns) (mu:sy-name symbol) value))
                    (mu:intern (core:%read-get-ns) (mu:sy-name symbol) value)))
            (mu:ns-find (core:%read-get-ns) (mu:sy-name symbol)))
           ((:lambda (ns name)
               (mu:intern
                (core:%read-get-ns)
                name
                value))
            (core:symbol-ns symbol)
            (mu:sy-name symbol)))))
