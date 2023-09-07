;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
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
                                             (mu:nth (mu:fx-add 1 offset) pairs))
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
      (:if (core:functionp fn)
           (:if (core:listp list)
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
                  (mu:cons () list)))
                (core:raise list 'core:findl "not a list"))
           (core:raise fn 'core:findl "not a function"))))

(mu:intern core "findr"
   (:lambda (fn list)
     (core:findl fn (core:reverse list))))

;;;
;;; position
;;;
(mu:intern core "positionl"
   (:lambda (fn list)
     (:if (core:functionp fn)
          (:if (core:listp list)
               (:if list
                    ((:lambda (len)
                        (mu:car (mu:fix
                           (:lambda (loop)
                              (:if (core:listp loop)
                                   loop
                                   (:if (mu:fx-lt len loop)
                                        ()
                                        (:if (core:apply fn `(,(mu:nth loop list)))
                                             `(,loop)
                                             (mu:fx-add 1 loop)))))
                           0)))
                     (mu:length list))
                    ())
               (core:raise list 'core:positionl "not a list"))
          (core:raise fn 'core:positionl "not a function"))))

(mu:intern core "positionr"
   (:lambda (fn list)
      (:if (core:functionp fn)
           (:if (core:listp list)
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
                                         (mu:fx-sub loop 1)))))
                       (mu:fx-sub (mu:length list) 1))))
                (core:raise list 'core:positionr "not a list"))
           (core:raise fn 'core:positionr "not a function"))))

;;;
;;; fixed argument list functions
;;;
(mu:intern core "append"
   (:lambda (list)
      (:if (core:listp list)                 
           (core:foldr
            (:lambda (el acc)
               (:if (core:listp el)
                    (core:foldr mu:cons acc el)
                    el))
            ()
            list)
      (core:raise list 'core:append "not a list"))))
   
(mu:intern core "reverse"
   (:lambda (list)
      (:if (core:listp list)
           (core:foldl mu:cons () list)
           (core:raise list 'core:reverse "not a list"))))
   
(mu:intern core "dropl"
   (:lambda (list n)
      (:if (core:listp list)
         (:if (core:fixnump n)
              (mu:nthcdr n list)
              (core:raise n 'core:dropl "not a fixnum"))
         (core:raise list 'core:dropl "not a list"))))

(mu:intern core "dropr"
   (:lambda (list n)
      (:if (core:listp list)
           (:if (core:fixnump n)
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
                  (mu:length list)))
                (core:raise n 'core:dropr "not a fixnum"))
           (core:raise list 'core:dropr "not a list"))))

(mu:intern core "last"
   (:lambda (list)
      (:if (core:listp list)
           (mu:fix
            (:lambda (loop)
               (:if (core:null list)
                    loop
                    ((:lambda (cdr)
                        (:if (core:consp cdr)
                             cdr
                             loop))
                     (mu:cdr loop))))
            list)
           (core:raise list 'core:last "not a list"))))
