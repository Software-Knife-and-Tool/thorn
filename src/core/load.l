;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; loader
;;;
(mu:intern core :extern "load"
  (:lambda (source verbose)
    (core:raise-unless core:stringp source 'core:load "not a file designator")
    ((:lambda (ifs)
       (core:raise-unless core:streamp ifs 'core:load "cannot open input file")
       (:if verbose
            (core:format :t ";;; loading ~A~%" `(,source))
            ())
       (mu:fix
        (:lambda (loop)
          (:if (core:eof ifs)
               loop
               (core:with-exception
                (:lambda (exception)
                     (core:exceptf mu:err-out "load exception: ~A~%" () exception))
                (:lambda ()
                   ((:lambda (form)
                      (:if (core:eof ifs)
                           loop
                           (:if verbose
                                (core:format :t "~A~%" `(,form))
                                ()))
                      (core:null loop))
                    (mu:eval (core:compile (core:read ifs :t ()))))))))
        ()))
     (mu:open :file :input source))))
