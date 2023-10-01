;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  symbols
;;;
(mu:intern :core "boundp" mu:boundp)
(mu:intern :core "fboundp"
  (:lambda (symbol)
    (:if (core:boundp symbol)
         (core:functionp (mu:sy-val symbol))
         ())))

(mu:intern :core "uninternedp"
   (:lambda (sym)
      (core:null (core:symbol-ns sym))))

(mu:intern :core "keywordp"
   (:lambda (t)
      (mu:eq (core:type-of t) 'keyword)))

(mu:intern :core "symbolp"
   (:lambda (t)
      (core:%orf (core:%orf (core:keywordp t) (mu:eq () t))
                (mu:eq 'symbol (core:type-of t)))))

;;;
;;; utilities
;;;
(mu:intern :core "symbol-name" mu:sy-name)

(mu:intern :core "symbol-ns"
   (:lambda (sym)
      (mu:sy-ns sym)))                 

(mu:intern :core "symbol-value" mu:sy-val)

(mu:intern :core "keyword"
   (:lambda (name)
      (:if (core:stringp name)
           (:if (mu:fx-lt (mu:sv-len name) 8)
                (mu:keyword name)
                (core:%make-core-type "keyword" `(,(mu:cons :name name))))
           (core:raise name 'core:keyword "not a keyword name"))))

(mu:intern :core "intern"
   (:lambda (symbol value)
      (:if (core:uninternedp symbol)
           ((:lambda (sym)
               (:if sym
                    (:if (core:boundp sym)
                         symbol
                         (mu:intern (core:%read-get-ns) (mu:sy-name symbol) value))
                    (mu:intern (core:%read-get-ns) (mu:sy-name symbol) value)))
            (mu:ns-find (core:%read-get-ns) (mu:sy-name symbol)))
           ((:lambda (name)
               (mu:intern
                (core:%read-get-ns)
                name
                value))
            (mu:sy-name symbol)))))
