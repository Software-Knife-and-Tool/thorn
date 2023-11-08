;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; lists
;;;
(mu:intern :core "consp" (:lambda (t) (mu:eq :cons (mu:type-of t))))
(mu:intern :core "null" (:lambda (t) (mu:eq () t)))
(mu:intern :core "listp" (:lambda (t) (:if (core:consp t) :t (core:null t))))

(mu:intern :core "dotted-pair-p"
  (:lambda (list)
    (:if (core:consp list)
         (core:atom (mu:cdr list))
         ())))

;;;
;;; utilities
;;;
(mu:intern :core "%pair-list"
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
(mu:intern :core "findl"
   (:lambda (function list)
      (:if (core:functionp function)
           (:if (core:listp list)
                (mu:cdr
                 (mu:fix
                  (:lambda (loop)
                     (:if (mu:eq :t (mu:car loop))
                          loop
                          (:if (mu:cdr loop)
                               ((:lambda (el)
                                   (:if (core:apply function `(,el))
                                        (mu:cons :t el)
                                        (mu:cons () (mu:cdr (mu:cdr loop)))))
                                (mu:car (mu:cdr loop)))
                               loop)))
                  (mu:cons () list)))
                (core:raise list 'core:findl "not a list"))
           (core:raise function 'core:findl "not a function"))))

(mu:intern :core "findr"
   (:lambda (function list)
     (core:findl function (core:reverse list))))

;;;
;;; position
;;;
(mu:intern :core "positionl"
   (:lambda (function list)
     (:if (core:functionp function)
          (:if (core:listp list)
               (:if list
                    ((:lambda (length)
                        (mu:car (mu:fix
                           (:lambda (loop)
                              (:if (core:listp loop)
                                   loop
                                   (:if (mu:fx-lt length loop)
                                        ()
                                        (:if (core:apply function `(,(mu:nth loop list)))
                                             `(,loop)
                                             (core:1+ loop)))))
                           0)))
                     (mu:length list))
                    ())
               (core:raise list 'core:positionl "not a list"))
          (core:raise function 'core:positionl "not a function"))))

(mu:intern :core "positionr"
   (:lambda (function list)
      (:if (core:functionp function)
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
                                    (:if (core:apply function `(,(mu:nth loop list)))
                                         `(,loop)
                                         (core:1- loop)))))
                       (mu:fx-sub (mu:length list) 1))))
                (core:raise list 'core:positionr "not a list"))
           (core:raise function 'core:positionr "not a function"))))

;;;
;;; fixed argument list functions
;;;
(mu:intern :core "append"
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

(mu:intern :core "reverse"
   (:lambda (list)
      (:if (core:listp list)
           (core:foldl mu:cons () list)
           (core:raise list 'core:reverse "not a list"))))

(mu:intern :core "dropl"
   (:lambda (list n)
      (:if (core:listp list)
         (:if (core:fixnump n)
              (mu:nthcdr n list)
              (core:raise n 'core:dropl "not a fixnum"))
         (core:raise list 'core:dropl "not a list"))))

(mu:intern :core "dropr"
   (:lambda (list n)
      (:if (core:listp list)
           (:if (core:fixnump n)
                (core:foldl
                 mu:cons
                 ()
                 ((:lambda (length)
                     (core:foldl
                      (:lambda (el last)
                         (:if (mu:fx-lt (mu:length last) (mu:fx-sub length n))
                              (mu:cons el last)
                              last))
                      ()
                      list))
                  (mu:length list)))
                (core:raise n 'core:dropr "not a fixnum"))
           (core:raise list 'core:dropr "not a list"))))

(mu:intern :core "last"
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

;;;
;;; count
;;;
(mu:intern :core "count"
   (:lambda (list item)
     (:if (core:listp list)
          (core:foldl          
           (:lambda (elt acc)
              (:if (mu:eq elt item)
                   (core:1+ acc)
                   acc))
           0
           list)
          (core:raise list 'core:count "not a list"))))

;;;
;;; misc
;;;
(mu:intern :core "%fx-list-max"
   (:lambda (list)
      (core:foldl          
       (:lambda (elt acc)
         (:if (mu:fx-lt acc elt)
              elt
              acc))
         (mu:car list)
         list)))

(mu:intern :core "%fx-list-min"
   (:lambda (list)
      (core:foldl
       (:lambda (elt acc)
         (:if (mu:fx-lt acc elt)
              acc
              elt))
         (mu:car list)
         list)))

(mu:intern :core "%fx-max-le"
   (:lambda (max list)
      ((:lambda (min)
          (:if (mu:fx-lt max min)
               max
               (core:foldl     
                (:lambda (elt acc)
                  (:if (core:%andf (mu:fx-lt elt max) (mu:fx-lt acc elt))
                       elt
                       acc))
                min
                list)))
       (core:%fx-list-min list))))

(mu:intern :core "%fx-min-ge"
   (:lambda (min list)
      ((:lambda (max)
          (:if (mu:fx-lt max min)
               min
               (core:foldl          
                (:lambda (elt acc)
                  (:if (core:%andf (mu:fx-lt min elt) (mu:fx-lt elt acc))
                       elt
                       acc))
                max
                list)))
         (core:%fx-list-max list))))
