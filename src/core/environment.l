;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core environment
;;;
(mu:intern :core "envp"
   (:lambda (env)
      (:if (core:core-type-p env)
           (mu:eq 'env (core:type-of env))
           ())))

(mu:intern :core "make-env"
   (:lambda (bindings)
     (:if (core:listp bindings)
          ((:lambda (env)
             (core:mapc
              (:lambda (pair)
                ((:lambda (symbol value)
                   (:if (core:symbolp symbol)
                        (core:add-env env symbol value)
                        (core:raise symbol 'core:make-env "not a symbol")))
                 (mu:car pair)
                 (mu:cdr pair)))
              bindings)
             env)
           (core:%make-env))
          (core:raise symbol 'core:make-env "not a symbol"))))

(mu:intern :core "get-env"
   (:lambda (env symbol)
     (:if (core:symbolp symbol)
         (:if (core:envp env)
              ((:lambda (map)
                 (:if (mu:mp-has map symbol)
                      (mu:mp-get map symbol)
                      (core:raise symbol 'core:get-env "symbol not in map")))
               (core:%env-prop :symbols env))
              (core:raise env 'core:get-env "not an env"))
         (core:raise symbol 'core:get-env "not a symbol"))))

(mu:intern :core "add-env"
   (:lambda (env symbol value)
     (:if (core:symbolp symbol)
         (:if (core:envp env)
              ((:lambda (map)
                 (:if (mu:mp-has map symbol)
                      (core:raise symbol 'core:add-env "symbol already in map")
                      (mu:mp-add map symbol value)))
               (core:%env-prop :symbols env))
              (core:raise env 'core:add-env "not an env"))
         (core:raise symbol 'core:add-env "not a symbol"))))

(mu:intern :core "env-size"
   (:lambda (env)
     (:if (core:envp env)
          ((:lambda (map)
             (mu:mp-size map))
           (core:%env-prop :symbols env))
          (core:raise env 'core:env-size "not an env"))))

;;;
;;; implementation
;;;

(mu:intern :core "%make-env"
   (:lambda ()
     (core:%make-core-type "env"
          (core:%pair-list
           `(:symbols   ,(mu:make-mp))))))

(mu:intern :core "%env-prop"
   (:lambda (prop env)
      (:if (core:findl (:lambda (env-prop) (mu:eq prop env-prop)) '(:symbols))
           (mu:cdr (core:%core-type-ref prop env))
           (core:raise prop 'core:%env-prop "not an environment property"))))
