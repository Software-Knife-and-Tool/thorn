;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core namespaces
;;;
(mu:intern :core "%gen-anon-ns-counter"
   (mu:open :string :output "0"))

(mu:intern :core "%gen-anon-ns"
   (:lambda ()
      ((:lambda (counter)                 
          (mu:write (core:1+ counter) () core:%gen-anon-ns-counter)
          (mu:make-ns (mu:keyword (core:format () "~X" `(,counter)))))
       (mu:read (mu:open :string :input (mu:get-str core:%gen-anon-ns-counter)) () ()))))

(mu:intern :core "%make-namespace"
   (:lambda (name)
      (mu:intern core:%core-namespaces name (core:%gen-anon-ns))))
