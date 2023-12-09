;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  prelude namespace
;;;

;;;
;;; namespace functions
;;;
(mu:intern :prelude "namespacep"
   (:lambda (ns)
      (mu:eq (prelude:type-of ns) 'namespace)))

(mu:intern :prelude "make-namespace"
   (:lambda (name uses)
      (:if (prelude:stringp name)
           (:if (prelude:listp uses)
                (prelude:%make-type "namespc"
                  (prelude:%pair-list
                   `(:name    ,name
                     :intern  ,(mu:make-ns)
                     :extern  ,(mu:make-ns)
                     :ch-mac  ,(mu:make-ns)
                     :sy-mac  ,(mu:make-ns)
                     :uses    ,uses)))
                (prelude:raise uses 'prelude:make-namespace "not a uses list"))
           (prelude:raise name 'prelude:make-namespace "not a name string"))))

(mu:intern :prelude "%namespace-prop"
   (:lambda (prop namespace)
      (:if (prelude:findl
            (:lambda (lambda-prop)
               (mu:eq prop lambda-prop))
            '(:name :intern :extern :ch-mac :rd-mac :sy-mac :uses))
           (prelude:%type-ref prop namespace)
           (prelude:raise prop 'prelude:%namespace-prop "not a namespace property"))))

(mu:intern :prelude "intern"
   (:lambda (symbol value)          
      (:if (prelude:uninternedp symbol)
           ((:lambda (sym)
               (:if sym
                    (:if (prelude:boundp sym)
                         symbol
                         (mu:intern (prelude:%read-get-ns) (mu:sy-name symbol) value))
                    (mu:intern (prelude:%read-get-ns) (mu:sy-name symbol) value)))
            (mu:ns-find (prelude:%read-get-ns) (mu:sy-name symbol)))
           ((:lambda (name)
               (mu:intern
                (prelude:%read-get-ns)
                name
                value))
            (mu:sy-name symbol)))))
