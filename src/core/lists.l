;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; lists
;;;
(mu:intern core "consp" (:lambda (t) (mu:eq :cons (mu:type-of t))))
(mu:intern core "null" (:lambda (t) (mu:eq () t)))
(mu:intern core "listp" (:lambda (t) (:if (core:consp t) :t (core:null t))))

(mu:intern core "dotted-pair-p"
  (:lambda (obj)
    (:if (core:consp obj)
         (core:atom (mu:cdr obj))
         ())))

;;;
;;; utilities
;;;
(mu:intern core "%pair-list"
   (:lambda (pairs)
      ((:lambda (len)
          (:if (core:zerop len)
               ()
               (:if (core:zerop (mu:logand len 1))
                    (mu:cdr
                     (mu:fix
                      (:lambda (pair-state)
                         ((:lambda (nth alist)
                             (:if (core:zerop nth)
                                  pair-state
                                  ((:lambda (offset)
                                      (mu:cons offset
                                         (mu:cons
                                          (mu:cons (mu:nth offset pairs)
                                             (mu:nth (core:1+ offset) pairs))
                                          alist)))
                                   (mu:fx-sub nth 2))))
                          (mu:car pair-state)
                          (mu:cdr pair-state)))
                      (mu:cons len ())))
                    (core:raise pairs 'core:%pair-list "malformed pair list"))))
       (mu:length pairs))))

;;;
;;; finds
;;;
(mu:intern core "findl"
   (:lambda (fn list)
      (core:raise-unless core:%appliablep fn 'core:findl "not a function")
      (core:raise-unless core:listp list 'core:findl "not a list")
      (mu:cdr
       (mu:fix
        (:lambda (loop)
           (:if (mu:eq :t (mu:car loop))
               loop
               (:if (mu:cdr loop)
                    ((:lambda (el)
                        (:if (core:apply fn `(,el))
                             (mu:cons :t el)
                             (mu:cons () (mu:cdr (mu:cdr loop)))))
                     (mu:car (mu:cdr loop)))
                    loop)))
        (mu:cons () list)))))

(mu:intern core "findr"
   (:lambda (fn list)
     (core:raise-unless core:%appliablep fn 'core:findr "not a function")
     (core:raise-unless core:sequencep list 'core:findr "not a list")
     (core:findl fn (core:reverse list))))

;;;
;;; position
;;;
(mu:intern core "positionl"
   (:lambda (fn list)
     (core:raise-unless core:%appliablep fn 'core:positionl "not a function")
     (core:raise-unless core:listp list 'core:positionl "not a list")
     (:if (core:null list)
          ()
          ((:lambda (len)
             (mu:car
              (mu:fix
               (:lambda (loop)
                 (:if (core:listp loop)
                      loop
                      (:if (mu:fx-lt len loop)
                           ()
                           (:if (core:apply fn `(,(mu:nth loop list)))
                                `(,loop)
                                (core:1+ loop)))))
               0)))
           (mu:length list)))))

(mu:intern core "positionr"
   (:lambda (fn list)
     (core:raise-unless core:%appliablep fn 'core:positionr "not a function")
     (core:raise-unless core:listp list 'core:positionr "not a list")
     (:if (core:null list)
          ()
          (mu:car
           (mu:fix
            (:lambda (loop)
              (:if (core:listp loop)
                   loop
                   (:if (mu:fx-lt loop 0)
                        ()
                        (:if (core:apply fn `(,(mu:nth loop list)))
                             `(,loop)
                             (core:1- loop)))))
            (core:1- (mu:length list)))))))

;;;
;;; fixed argument list functions
;;;
(mu:intern core "append"
   (:lambda (list)
     (core:raise-unless core:listp list 'core:append "not a list")
     (core:foldr
      (:lambda (el acc)
        (:if (core:listp el)
          (core:foldr mu:cons acc el)
          el))
      ()
      list)))

(mu:intern core "reverse"
  (:lambda (list)
    (core:raise-unless core:listp list 'core:reverse "not a list")
    (core:foldl mu:cons () list)))

(mu:intern core "dropl"
   (:lambda (list n)
      (core:raise-unless core:listp list 'core:dropl "not a list")
      (core:raise-unless core:fixnump n 'core:dropl "not a fixnum")
      (mu:nthcdr n list)))

(mu:intern core "dropr"
   (:lambda (list n)
      (core:raise-unless core:listp list 'core:dropr "not a list")
      (core:raise-unless core:fixnump n 'core:dropr "not a fixnum")
      (core:foldl
        mu:cons
        ()
        ((:lambda (len)
            (core:foldl
              (:lambda (el last)
                 (:if (mu:fx-lt (mu:length last) (mu:fx-sub (mu:length list) n))
                      (mu:cons el last)
                      last))
                 ()
              list))
         (mu:length list)))))

(mu:intern core "last"
   (:lambda (list)
     (core:raise-unless core:listp list 'core:last "not a list")
     (mu:fix
      (:lambda (loop)
        (:if (core:null list)
             loop
             ((:lambda (cdr)
                (:if (core:consp cdr)
                     cdr
                     loop))
              (mu:cdr loop))))
      list)))
