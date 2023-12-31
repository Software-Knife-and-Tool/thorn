;;;
;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  thorn runtime environment
;;;

#|
(core:load "/opt/thorn/thorn/thorn/thorn.l" ())

(mu:eval (core:compile '(defconst thorn::require-path "/opt/thorn/lib")))

(core:in-namespace (mu:ns "user" (mu:map-ns "thorn")))

(:if (core:zerop (system:system "stat ~/.thorn > /dev/null 2>&1"))
     (core:load "~/.thorn" () ())
     ())

(core:mapc
 (:lambda (fn) (core:load fn :t ()))
 (mu:read (mu:open :string :input (system:getenv "THORN_LOAD_LIST"))))

(mu:intern (mu:find-ns "thorn") :intern "thorn-version" "0.0.1")

(thorn:repl (mu:find-ns "user"))
|#
