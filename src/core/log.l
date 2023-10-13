;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; logging
;;;
(mu:intern :core "%logf"
   (:lambda (stream fmt args)
      (core:apply core:format `(,stream ,fmt ,args))))

(mu:intern :core "logf"
   (:lambda (env fmt args)                 
      ((:lambda (stream)
          (core:apply core:format `(,stream ,fmt ,args))))
      (core:env-prop :log env)))

(mu:intern :core "%open-log-file"
   (:lambda (file-name env)
      (mu:open :file :output file-name)))

;;;
;;; exception flavors
;;;
(mu:intern :core "log-error"
   (:lambda (fmt object env)
      (core:logf env fmt `(,object))
      (core:error fmt object)))

(mu:intern :core "log-raise"
   (:lambda (object source reason env)
      (core:logf env fmt `(,object ,source ,reason ,env))
      (core:raise object source reason)))

(mu:intern :core "log-warn"
   (:lambda (obj msg env)
      (core:logf env ";;; warning: ~S ~A~%" `(,obj ,msg))
      obj))
