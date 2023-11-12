;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core namespace
;;;

;;;
;;; namespace functions
;;;
(mu:intern :core "namespacep"
   (:lambda (ns)
      (mu:eq (core:type-of ns) 'namespace)))

(mu:intern :core "make-namespace"
   (:lambda (name uses)
      (:if (core:stringp name)
           (:if (core:listp uses)
                (core:%make-core-type "nmspace"
                  (core:%pair-list
                   `(:name    ,name
                     :intern  ,(mu:make-ns)
                     :extern  ,(mu:make-ns)
                     :ch-mac  ,(mu:make-ns)
                     :sy-mac  ,(mu:make-ns)
                     :uses    ,uses)))
                (core:raise uses 'make-namespace "not a uses list"))
           (core:raise name 'make-namespace "not a name string"))))

(mu:intern :core "%namespace-prop"
   (:lambda (prop namespace)
      (:if (core:findl
            (:lambda (lambda-prop)
               (mu:eq prop lambda-prop))
            '(:name :intern :extern :ch-mac :rd-mac :sy-mac :uses))
           (core:%core-type-ref prop namespace)
           (core:raise prop 'namespace-prop "not a namespace property"))))

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
