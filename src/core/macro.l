;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; macro definition, compilation, and expansion
;;;

;;;
;;; Macro definitions are compiled like lambdas (though eventually with a slighlty
;;; different lambda list including &whole).
;;;
;;; Macro calls are expanded at compile time.
;;;
;;; Common Lisp adds an environment argument here for lexical macro expanders.
;;;

;;;
;;; macro environment
;;;
(mu:intern :core "%map-macro-symbol-ns"
   (:lambda (symbol)
      ((:lambda (ns-name)
          (mu:ns-find core:%macro-ns% ns-name))
       (mu:sy-name (mu:sy-ns symbol)))))

(mu:intern :core "%make-macro-ns"
   (:lambda (ns)
      ((:lambda (macro-ns)
          (mu:intern core:%macro-ns% (mu:sy-name ns) macro-ns)
          macro-ns)
       (core:%gen-anon-ns))))    

(mu:intern :core "%macro-env"
   (:lambda (symbol macro-fn)
      ((:lambda (macro-ns symbol-ns)
          (:if macro-ns
               (mu:intern macro-ns symbol-ns macro-fn)
               (mu:intern (core:%make-macro-ns symbol-ns) (mu:sy-name symbol) macro-fn))
          symbol)
       (core:%map-macro-symbol-ns symbol)
       (mu:sy-ns symbol))))

(mu:intern :core "%find-macro"
   (:lambda (symbol env)
      ((:lambda (macro-ns symbol-ns symbol-name)
          (:if macro-ns
               (mu:ns-find (mu:sy-ns macro-ns) symbol-name)
               ()))
       (mu:ns-find
        core:%macro-ns%
        (mu:sy-name (mu:sy-ns symbol)))
       (mu:sy-name (mu:sy-ns symbol))
       (mu:sy-name symbol))))

(mu:intern :core "%compile-macro-call"
   (:lambda (macro-symbol args env)
      (core:%compile
       (core:macroexpand `(,macro-symbol ,@args) env)
       env)))

;;;
;;; macroexpand-1 expands a macro call once.
;;;
(mu:intern :core "macroexpand-1"
   (:lambda (form env)
     (:if (core:consp form)
          ((:lambda (fn-symbol args)
             (:if (mu:eq :symbol (mu:type-of fn-symbol))
                  ((:lambda (macro-fn)
                      (:if macro-fn
                           (core:apply macro-fn (mu:eval (core:%arg-list args)))
                           form))
                   (core:macro-function fn-symbol env))
                  form))
           (mu:car form)
           (mu:cdr form))
          form)))

;;;
;;; macroexpand: expand a form until it is no longer a macro call
;;;
(mu:intern :core "macroexpand"
   (:lambda (form env)
     ((:lambda (expanded)
          (:if (mu:eq form expanded)
             expanded
             (core:macroexpand expanded env)))
       (core:macroexpand-1 form env))))

;;;
;;; macro-function: map a symbol to a macro function
;;;
;;; acts as a predicate
;;;
(mu:intern :core "macro-function"
   (:lambda (symbol env)
      (:if (mu:eq :symbol (mu:type-of symbol))
           ((:lambda (macro)
               (:if macro
                    (mu:sy-val macro)
                    ()))
            (core:%find-macro symbol env))
          ())))
