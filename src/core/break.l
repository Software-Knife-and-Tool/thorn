;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; debugging
;;;
(mu:intern :core "%debug-map-symbol"
   (:lambda (value)
      (:if (mu:boundp 'core:%debug-core-symbols%)
           ()
           (mu:intern :core "%debug-core-symbols%" (mu:ns-syms :list :core)))
      (mu:fix
       (:lambda (list)
          (:if (core:consp list)
               ((:lambda (symbol)
                   (:if (mu:eq (mu:sy-val symbol) value)
                        symbol
                        (mu:cdr list)))
                (mu:car list))
               list))
       core:%debug-core-symbols%)))

(mu:intern :core "%format-mapped-symbol"
   (:lambda (value)
      ((:lambda (symbol)
          (core:%orf (mu:nth 0 symbol) (mu:nth 1 symbol)))
       (core:%debug-map-symbol value))))

;;;
;;; break loop
;;;
(mu:intern :core "break"
   (:lambda (except)
     (core:format :t ";;; entering break loop with exception~%" ())
     (core:exceptf mu:std-out ";;; ~A on ~A by ~S, ~A~%" () except)
     (core:format :t ";;; :h for commands~%" ())
     (mu:fix
      (:lambda (loop)
         (:if (core:eof :t)
              ()
              ((:lambda (form)
                  (:if (mu:eq form :h)
                       ((:lambda ()
                           (core:format :t "break help:~%" ())
                           (core:format :t ":d - describe exception value~%" ())
                           (core:format :t ":e - print the full exception~%" ())
                           (core:format :t ":r - return from break~%" ())
                           (core:format :t ":x - exit process~%" ())
                           (core:null loop)))
                       (:if (mu:eq form :d)
                            ((:lambda (value)
                                (core:describe (mu:cdr value) :t)
                                (core:null loop))
                             (core:%except-prop :value except))
                            (:if (mu:eq form :x)
                                 (mu:exit 0)
                                 (:if (mu:eq form :e)
                                      ((:lambda ()
                                          (core:exceptf mu:std-out "exception: ~A on ~A by ~S, ~A~%" :t except)
                                          (core:null loop)))
                                      (:if (mu:eq form :r)
                                           loop
                                           ((:lambda ()
                                               (core:format :t ";;; unrecognized debugger command: h for help~%" ())
                                               (core:null loop)))))))))
                   ((:lambda ()
                       (core:format :t "core:debug> " ())
                       (mu:flush mu:std-out)
                       (core:read :t () ()))))))
     ())))
