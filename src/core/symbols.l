;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core symbols
;;;
(mu:intern core::ns :extern "symbol-name" mu:sy-name)
(mu:intern core::ns :extern "symbol-ns" mu:sy-ns)
(mu:intern core::ns :extern "symbol-value" mu:sy-val)

(mu:intern core::ns :extern "keywordp"
   (:lambda (t)                 
      (mu:eq (core:type-of t) :keyword)))

(mu:intern core::ns :extern "symbolp"
   (:lambda (t)
      ((:lambda (type)
          (:if (mu:eq :null type)
               :t
               (:if (mu:eq :keyword type)
                    :t
                    (mu:eq :symbol type))))
       (core:type-of t))))

(mu:intern core::ns :extern "keyword"
   (:lambda (name)
      (core:raise-unless core:stringp name 'core:keyword "not a keyword name")
      (:if (mu:fx-lt (mu:sv-len name) 8)
           (mu:keyword name)
           (core::make-ctype "keyword" `(,(mu:cons :name name))))))

(mu:intern core::ns :intern "intern"
   (:lambda (symbol value env)
      (:if (core:uninternedp symbol)
           ((:lambda (sym)
               (:if sym
                    (:if (core:boundp sym)
                         (core:raise sym 'core::intern "symbol already bound")
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
