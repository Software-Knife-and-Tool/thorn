;;;  SPDX-FileCopyrightText: Copyright 2017 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; return the elapsed process time in usec
;;; from applying fn to args
;;;
(mu:intern :mu "%tdelta"
  (:lambda (fn args)
    ((:lambda (start stop)
       (mu:fx-sub stop start))
       (sys:run-us)
       ((:lambda ()
          (mu:apply fn args)
          (sys:run-us))))))

;;;
;;; return an vector of storage consumed
;;; by applying fn to args
;;;
(mu:intern :mu "%sdelta"
  (:lambda (fn args)
    ((:lambda (start-vec stop-vec)
       (mu:fix
        (:lambda (loop)
          (:if (mu:eq :vector (mu:type-of loop))
               loop
               ((:lambda (list n)
                  ((:lambda (offset)
                     (:if (mu:fx-lt offset (mu:sv-len start-vec))
                          ((:lambda (type total alloc in-use)
                             ((:lambda (type-stats)
                                (mu:cons `(,@list ,@type-stats) (mu:fx-add 1 n)))
                              `(,type ,total ,alloc ,in-use)))
                           (mu:sv-ref start-vec (mu:fx-add offset 0))
                           (mu:fx-sub (mu:sv-ref stop-vec (mu:fx-add offset 1)) (mu:sv-ref start-vec (mu:fx-add offset 1)))
                           (mu:fx-sub (mu:sv-ref stop-vec (mu:fx-add offset 2)) (mu:sv-ref start-vec (mu:fx-add offset 2)))
                           (mu:fx-sub (mu:sv-ref stop-vec (mu:fx-add offset 3)) (mu:sv-ref start-vec (mu:fx-add offset 3))))
                          (mu:vector :t list)))
                         (mu:fx-mul n 4)))
                 (mu:car loop)
                 (mu:cdr loop))))
         (mu:cons () 1)))
     (mu:hp-stat)
     ((:lambda ()
         (mu:apply fn args)
         (mu:hp-stat))))))
