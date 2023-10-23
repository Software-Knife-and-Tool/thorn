;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; logging
;;;
(mu:intern :core "%logf"
   (:lambda (stream format-satring arg-list)
      (core:apply core:format `(,stream ,format-string ,arg-list))))

(mu:intern :core "logf"
   (:lambda (env format-string arg-list)
      ((:lambda (stream)
          (core:apply core:format `(,stream ,format-string ,arg-list))))
      (core:%env-prop :log env)))

(mu:intern :core "%open-log-file"
   (:lambda (file-name env)
      (mu:open :file :output file-name)))

;;;
;;; exception flavors
;;;
(mu:intern :core "log-error"
   (:lambda (format-string value env)
      (core:logf env format-string `(,value))
      (core:error format-string value)))

(mu:intern :core "log-raise"
   (:lambda (format-string value source reason env)
      (core:logf env format-string `(,value ,source ,reason ,env))
      (core:raise value source reason)))

(mu:intern :core "log-warn"
   (:lambda (value message env)
      (core:logf env ";;; warning: ~S ~A~%" `(,value ,messagee))
      value))
