;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; symbol table
;;;
(mu:intern :core "symbols"
   (:lambda (file-name)
      ((:lambda (stream)
          (core:mapc
           (:lambda (symbol)
              (:if (core:null (mu:sy-ns symbol))
                   (core:format stream "~A~%" `(,(mu:sy-name symbol)))
                   (core:format stream "core:~A~%" `(,(mu:sy-name symbol)))))
           (mu:ns-syms :list :core))
          (core:mapc
           (:lambda (symbol)
              (:if (core:null (mu:sy-ns symbol))
                   (core:format stream "~A~%" `(,(mu:sy-name symbol)))
                   (core:format stream "mu:~A~%" `(,(mu:sy-name symbol)))))
           (mu:ns-syms :list :mu)))
       (mu:open :file :output file-name))
      :t))
