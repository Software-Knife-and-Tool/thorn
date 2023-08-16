;;;  Spdx-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; cross reference
;;;
(mu:untern core :intern "core-symbols")

(mu:intern core :intern "xref"
   (:lambda (file-name)
      ((:lambda (stream symbols)
          (core:mapc
           (:lambda (symbol)
              (:if (mu:boundp symbol)
                   (core:format stream "~A~T~A~T~S~%"
                                `(,(mu:sy-name symbol)
                                  ,(core:type-of (mu:sy-val symbol))
                                  ,(mu:sy-val symbol)))
                   (core:format stream "unbound~Tunbound~T~A~%" `(,(mu:sy-name symbol)))))
           (mu:nth 0 symbols))
          (core:mapc
           (:lambda (symbol)
              (:if (mu:boundp symbol)
                   (core:format stream ":~A~T~A~T~S~%"
                                `(,(mu:sy-name symbol)
                                  ,(core:type-of (mu:sy-val symbol))
                                  ,(mu:sy-val symbol)))
                   (core:format stream "unbound~Tunbound~T:~A~%" `(,(mu:sy-name symbol)))))
           (mu:nth 1 symbols)))
       (mu:open :file :output file-name)
       `(,(mu:ns-ext core) ,(mu:ns-int core)))
      :t))
