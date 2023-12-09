;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; lists
;;;
(mu:intern :prelude "consp" (:lambda (t) (mu:eq :cons (mu:type-of t))))
(mu:intern :prelude "null" (:lambda (t) (mu:eq () t)))
(mu:intern :prelude "listp" (:lambda (t) (:if (prelude:consp t) :t (prelude:null t))))

(mu:intern :prelude "dotted-pair-p"
  (:lambda (list)
    (:if (prelude:consp list)
         (prelude:atom (mu:cdr list))
         ())))

;;;
;;; utilities
;;;
(mu:intern :prelude "%pair-list"
   (:lambda (pairs)
      ((:lambda (len)
          (:if (prelude:zerop len)
               ()
               (:if (prelude:zerop (mu:logand len 1))
                    (mu:cdr
                     (mu:fix
                      (:lambda (pair-state)
                         ((:lambda (nth alist)
                             (:if (prelude:zerop nth)
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
                    (prelude:raise pairs 'prelude:%pair-list "malformed pair list"))))
       (mu:length pairs))))

;;;
;;; finds
;;;
(mu:intern :prelude "findl"
   (:lambda (function list)
      (:if (prelude:functionp function)
           (:if (prelude:listp list)
                (mu:cdr
                 (mu:fix
                  (:lambda (loop)
                     (:if (mu:eq :t (mu:car loop))
                          loop
                          (:if (mu:cdr loop)
                               ((:lambda (el)
                                   (:if (prelude:apply function (mu:cons el ()))
                                        (mu:cons :t el)
                                        (mu:cons () (mu:cdr (mu:cdr loop)))))
                                (mu:car (mu:cdr loop)))
                               loop)))
                  (mu:cons () list)))
                (prelude:raise list 'prelude:findl "not a list"))
           (prelude:raise function 'prelude:findl "not a function"))))

(mu:intern :prelude "findr"
   (:lambda (function list)
     (prelude:findl function (prelude:reverse list))))

;;;
;;; position
;;;
(mu:intern :prelude "positionl"
   (:lambda (function list)
     (:if (prelude:functionp function)
          (:if (prelude:listp list)
               (:if list
                    ((:lambda (length)
                        (mu:car (mu:fix
                           (:lambda (loop)
                              (:if (prelude:listp loop)
                                   loop
                                   (:if (mu:fx-lt length loop)
                                        ()
                                        (:if (prelude:apply function (mu:cons (mu:nth loop list) ()))
                                             (mu:cons loop ())
                                             (prelude:1+ loop)))))
                           0)))
                     (mu:length list))
                    ())
               (prelude:raise list 'prelude:positionl "not a list"))
          (prelude:raise function 'prelude:positionl "not a function"))))

(mu:intern :prelude "positionr"
   (:lambda (function list)
      (:if (prelude:functionp function)
           (:if (prelude:listp list)
                (:if (prelude:null list)
                     ()
                     (mu:car
                      (mu:fix
                       (:lambda (loop)
                          (:if (prelude:listp loop)
                               loop
                               (:if (mu:fx-lt loop 0)
                                    ()
                                    (:if (prelude:apply function (mu:cons (mu:nth loop list) ()))
                                         (mu:cons loop ())
                                         (prelude:1- loop)))))
                       (mu:fx-sub (mu:length list) 1))))
                (prelude:raise list 'prelude:positionr "not a list"))
           (prelude:raise function 'prelude:positionr "not a function"))))

;;;
;;; fixed argument list functions
;;;
(mu:intern :prelude "append"
   (:lambda (list)
      (:if (prelude:listp list)
           (prelude:foldr
            (:lambda (el acc)
               (:if (prelude:listp el)
                    (prelude:foldr mu:cons acc el)
                    el))
            ()
            list)
      (prelude:raise list 'prelude:append "not a list"))))

(mu:intern :prelude "reverse"
   (:lambda (list)
      (:if (prelude:listp list)
           (prelude:foldl mu:cons () list)
           (prelude:raise list 'prelude:reverse "not a list"))))

(mu:intern :prelude "dropl"
   (:lambda (list n)
      (:if (prelude:listp list)
         (:if (prelude:fixnump n)
              (mu:nthcdr n list)
              (prelude:raise n 'prelude:dropl "not a fixnum"))
         (prelude:raise list 'prelude:dropl "not a list"))))

(mu:intern :prelude "dropr"
   (:lambda (list n)
      (:if (prelude:listp list)
           (:if (prelude:fixnump n)
                (prelude:foldl
                 mu:cons
                 ()
                 ((:lambda (length)
                     (prelude:foldl
                      (:lambda (el last)
                         (:if (mu:fx-lt (mu:length last) (mu:fx-sub length n))
                              (mu:cons el last)
                              last))
                      ()
                      list))
                  (mu:length list)))
                (prelude:raise n 'prelude:dropr "not a fixnum"))
           (prelude:raise list 'prelude:dropr "not a list"))))

(mu:intern :prelude "last"
   (:lambda (list)
      (:if (prelude:listp list)
           (mu:fix
            (:lambda (loop)
               (:if (prelude:null list)
                    loop
                    ((:lambda (cdr)
                        (:if (prelude:consp cdr)
                             cdr
                             loop))
                     (mu:cdr loop))))
            list)
           (prelude:raise list 'prelude:last "not a list"))))

;;;
;;; count
;;;
(mu:intern :prelude "count"
   (:lambda (item list)
     (:if (prelude:listp list)
          (prelude:foldl          
           (:lambda (elt acc)
              (:if (mu:eq elt item)
                   (prelude:1+ acc)
                   acc))
           0
           list)
          (prelude:raise list 'prelude:count "not a list"))))

;;;
;;; misc
;;;
(mu:intern :prelude "%fx-list-max"
   (:lambda (list)
      (prelude:foldl          
       (:lambda (elt acc)
         (:if (mu:fx-lt acc elt)
              elt
              acc))
         (mu:car list)
         list)))

(mu:intern :prelude "%fx-list-min"
   (:lambda (list)
      (prelude:foldl
       (:lambda (elt acc)
         (:if (mu:fx-lt acc elt)
              acc
              elt))
         (mu:car list)
         list)))

(mu:intern :prelude "%fx-max-le"
   (:lambda (max list)
      ((:lambda (min)
          (:if (mu:fx-lt max min)
               max
               (prelude:foldl     
                (:lambda (elt acc)
                  (:if (prelude:%andf (mu:fx-lt elt max) (mu:fx-lt acc elt))
                       elt
                       acc))
                min
                list)))
       (prelude:%fx-list-min list))))

(mu:intern :prelude "%fx-min-ge"
   (:lambda (min list)
      ((:lambda (max)
          (:if (mu:fx-lt max min)
               min
               (prelude:foldl          
                (:lambda (elt acc)
                  (:if (prelude:%andf (mu:fx-lt min elt) (mu:fx-lt elt acc))
                       elt
                       acc))
                max
                list)))
         (prelude:%fx-list-max list))))
