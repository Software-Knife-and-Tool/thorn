;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; loader
;;;
(mu:intern core "load"
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

;;;
;;; file compiler
;;;
(mu:intern core "compile-file"
  (:lambda (source dest verbose)
    (core:raise-unless core:stringp source 'core:compile-file "not a file designator")
    ((:lambda (ifs ofs)
       (core:raise-unless core:streamp ifs 'core:compile-file "cannot open input file")
       (core:raise-unless core:streamp ofs 'core:compile-file "cannot open output file")
       (:if verbose
            (core:format :t ";;; compiling ~A~%" `(,source))
            ())
       (core:format ofs "(core:%load-fasl ~S #(:t ~%" `(source)) 
       (mu:fix
        (:lambda (loop)
          (:if (core:eof ifs)
               loop
               (core:with-exception
                (:lambda (ex)
                   (core:exceptf mu:err-out "compile exception: ~A on ~A by ~S from ~A~%" () ex)
                   loop)
                (:lambda ()
                   ((:lambda (form)
                       (:if (core:eof ifs)
                           loop
                           ((:lambda ()
                               (:if verbose
                                    (core:format :t "form: ~S~%" `(,form))
                                    ())
                               (core:format ofs " ~S~%" `(,(mu:compile form)))
                               (core:null loop)))))
                    (core:read ifs :t ()))))))
        ())
       (core:format ofs ")" ())
       (mu:close ifs)
       (mu:close ofs))
     (mu:open :file :input source)
     (mu:open :file :output dest))))
