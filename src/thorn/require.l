;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; thorn require
;;;
(defconst thorn::require-ns (mu:ns (symbol-name (gensym)) (env:find-ns "thorn")))
(core:in-namespace thorn::require-ns)

(defmacro thorn:require-lib (lib-sym)
  (errorp-unless symbolp lib-sym "require-lib: not a symbol")
  (let ((lib (symbol-name lib-sym)))
    (if (mu:ns-find thorn::require-ns :extern lib)
      ()
      (let ((lib-path (core:format () "~A/~A:~A.l" (list thorn::require-path "eko" lib))))
        (mu:intern thorn::require-ns :extern lib)
        (core:load lib-path () ())))))

(defmacro thorn:require-lib-verbose (lib-sym)
  (errorp-unless symbolp lib-sym "require-lib not a symbol")
  (let ((lib (symbol-name lib-sym)))
    (if (mu:ns-find thorn::require-ns :extern lib)
      ()
      (let ((lib-path (core:format () "~A/~A:~A.l" (list thorn::require-path "eko" lib))))
        (mu:intern thorn::require-ns :extern lib)
        (core:load lib-path :t ())))))

