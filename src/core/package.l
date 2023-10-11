;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core packages
;;;

;;;
;;; namespace components
;;;
(mu:intern :core "%gen-anon-ns-counter"
   (mu:open :string :output "0"))

(mu:intern :core "%gen-anon-ns"
   (:lambda ()
      ((:lambda (counter)                 
          (mu:write (core:1+ counter) () core:%gen-anon-ns-counter)
          (mu:make-ns (mu:keyword (core:format () "%~X" `(,counter)))))
       (mu:read (mu:open :string :input (mu:get-str core:%gen-anon-ns-counter)) () ()))))

;;;
;;; package functions
;;;
(mu:intern :core "packagep"
   (:lambda (package)
      (mu:eq (core:type-of package) 'package)))

(mu:intern :core "make-package"
   (:lambda (name uses)
      (:if (core:stringp name)
           (:if (core:listp uses)           
                (core:%make-core-type "package"
                  (core:%pair-list
                   `(:name    ,name
                     :intern  ,(core:%gen-anon-ns)
                     :extern  ,(core:%gen-anon-ns)
                     :ch-mac  ,(core:%gen-anon-ns)
                     :rd-mac  ,(core:%gen-anon-ns)
                     :sy-mac  ,(core:%gen-anon-ns)
                     :uses    ,uses)))
                (core:raise uses 'make-package "not a uses list"))
           (core:raise name 'make-package "not a name string"))))
 
(mu:intern :core "%package-prop"
   (:lambda (prop package)
      (:if (core:findl
            (:lambda (lambda-prop)
               (mu:eq prop lambda-prop))
            '(:name :intern :extern :ch-mac :rd-mac :sy-mac :uses))
           (core:%core-type-ref prop package)
           (core:raise prop 'package-prop "not a package property"))))

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
