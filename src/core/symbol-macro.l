;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; symbol macros
;;;
(mu:intern core "in-namespace"
   (:lambda (ns)
     (:if (core:namespacep ns)
          ((:lambda ()
              (mu:make-ns ns)
              (core:%read-set-ns ns)
              ns))
          (core:raise ns 'core:in-namespace "not a namespace"))))

(mu:intern core "%read-get-ns"
   (:lambda ()
     ((:lambda (name)
        (mu:write name () core:%reader-ns-stream)
        (mu:map-ns name))
      (core:get-output-stream-string core:%reader-ns-stream))))

(mu:intern core "%read-set-ns"
   (:lambda (ns)
      (core:get-output-stream-string core:%reader-ns-stream)
      (mu:write (mu:ns-name ns) () core:%reader-ns-stream)))

;;;
;;; symbol readers
;;;
(mu:intern core "%read-keywordp"
  (:lambda (name)
      (mu:eq #\: (core:schar name 0))))

(mu:intern core "%read-symbol-scope"
  (:lambda (name)
     ((:lambda (colon)
        (:if (core:null colon)
             :extern
             (:if (mu:eq #\: (core:schar name (mu:fx-add 1 colon)))
                  :intern
                  :extern)))
        (core:string-position #\: name))))

(mu:intern core "%read-symbol-ns"
  (:lambda (name)
    ((:lambda (colon)
       (:if (core:null colon)
            ()
            (mu:map-ns (core:substr name 0 (mu:fx-sub colon 1)))))
     (core:string-position #\: name))))

(mu:intern core "%read-symbol-name"
   (:lambda (name)
     ((:lambda (colon)
        (:if (core:null colon)
             name
             (core:substr name
                          (:if (mu:eq #\: (core:schar name (mu:fx-add 1 colon)))
                               (mu:fx-add 2 colon)
                               (mu:fx-add 1 colon))
                          (:if (mu:eq #\: (core:schar name (mu:fx-add 1 colon)))
                               (mu:fx-add colon (mu:fx-sub (core:length name) colon))
                               (mu:fx-add colon (mu:fx-sub (core:length name) (mu:fx-add 1 colon)))))))
     (core:string-position #\: name))))

(mu:intern core "%read-symbol"
   (:lambda (symbol)
     (:if (core:%read-keywordp symbol)
          (core:keyword (core:%read-symbol-name symbol))
          ((:lambda (ns name)
             (:if (core:null ns)
                  (mu:untern () name)
                  (mu:untern ns name)))
           (core:%read-symbol-ns symbol)
           (core:%read-symbol-name symbol)))))

;;;
;;; symbol macros
;;;
(mu:intern core "%symbol-macro-expand"
   (:lambda (symbol)
     (:if (core:uninternedp symbol)
          ((:lambda (sym)
             (:if sym
                  (mu:eval (core:symbol-value sym))
                  symbol))
           (core:%get-symbol-macro symbol))
          symbol)))

(mu:intern core "%get-symbol-macro"
   (:lambda (symbol)
      (:if (mu:eq :symbol (mu:type-of symbol))
           (mu:ns-find core:%symbol-macro-ns% (core:symbol-name symbol))
           (core:raise symbol 'get-symbol-macro-form "not a symbol"))))

(mu:intern core "define-symbol-macro"
   (:lambda (symbol form)
      (:if (mu:eq :symbol (mu:type-of symbol))
           (mu:intern core:%symbol-macro-ns% (mu:sy-name symbol) form)
           (core:raise symbol 'core:define-symbol-macro "not a symbol"))))

;;; (core:define-symbol-macro 't :t)
;;; (core:define-symbol-macro 'nil :nil)
