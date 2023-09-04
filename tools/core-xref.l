;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; cross reference
;;;
(mu:intern core "xref"
   (:lambda (file-name)
      ((:lambda (stream)
          (core:mapc
           (:lambda (symbol)
              (core:warn symbol "symbol:")
              (:if (mu:boundp symbol)
                   (core:format stream "~A~T~A~T~S~%"
                                `(,(mu:sy-name symbol)
                                  ,(core:type-of (mu:sy-val symbol))
                                  ,(mu:sy-val symbol)))
                   (core:format stream "unbound~Tunbound~T~A~%" `(,(mu:sy-name symbol)))))
           (mu:ns-syms core)))
       (mu:open :file :output file-name))
      :t))
