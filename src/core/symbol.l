;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  symbol predicates
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
;;; symbol components
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
           (core:raise name 'core:keyword "not a legal name"))))

;;;
;;; utilities
;;;
(mu:intern :core "%gensym-counter"
   (mu:open :string :output "0"))

(mu:intern :core "gensym"
   (:lambda ()
      ((:lambda (counter)
          (mu:write (core:1+ counter) () core:%gensym-counter)
          (mu:make-sy (core:format () "g~X" `(,counter))))
       (core:read-string (mu:get-str core:%gensym-counter) () ()))))

(mu:intern :core "genkeyword"
   (:lambda (prefix)
      (:if (core:%orf (core:null prefix) (core:charp prefix))
           ((:lambda (counter)
              (mu:write (core:1+ counter) () core:%gensym-counter)
              ((:lambda (keyword-name)
                 (mu:read (mu:open :string :input keyword-name) () ()))
               (:if (core:charp prefix)
                    (core:format () ":~A~X" `(,prefix ,counter))
                    (core:format () ":<~X>" `(,counter)))))
            (core:read-string (mu:get-str core:%gensym-counter) () ()))
           (core:raise name 'core:genkeyword "not a legal name"))))
