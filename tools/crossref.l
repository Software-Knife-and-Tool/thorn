;;;  Spdx-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; cross reference
;;;
(mu:untern core :intern "core-symbols")

(mu:intern core :intern "crossref"
   (:lambda (file-name)
      (:if (mu:boundp 'core::core-symbols)
           ()
           (mu:intern core :intern "core-symbols" `(,(mu:ns-ext core) ,(mu:ns-int core))))
      ((:lambda (stream)
         (core:mapc
          (:lambda (symbol)
             (:if (mu:boundp symbol)
                  (core:format stream "~A~T~A~T~S~%" `(,(mu:sy-name symbol) ,(core:type-of (mu:sy-val symbol)) ,(mu:sy-val symbol)))
                  (core:format stream "unbound~Tunbound~T~A~%" `(,(mu:sy-name symbol)))))
          (mu:nth 0 core::core-symbols))
         (core:mapc
          (:lambda (symbol)
             (:if (mu:boundp symbol)
                  (core:format stream ":~A~T~A~T~S~%" `(,(mu:sy-name symbol) ,(core:type-of (mu:sy-val symbol)),(mu:sy-val symbol)))
                  (core:format stream "unbound~Tunbound~T:~A~%" `(,(mu:sy-name symbol)))))
          (mu:nth 1 core::core-symbols)))
       (mu:open :file :output file-name))
      file-name))
