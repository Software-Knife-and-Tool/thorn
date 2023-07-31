;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; lists
;;;
(mu:intern core::ns :extern "consp" (:lambda (t) (mu:eq :cons (mu:type-of t))))
(mu:intern core::ns :extern "null" (:lambda (t) (mu:eq () t)))
(mu:intern core::ns :extern "listp" (:lambda (t) (:if (core:consp t) :t (core:null t))))

(mu:intern core::ns :extern "dottedp"
  (:lambda (obj)
    (:if (core:consp obj)
         (core:atom (mu:cdr obj))
         ())))

;;;
;;; utilities
;;;
(mu:intern core::ns :intern "pair-list"
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
                    (core:raise pairs 'core::pair-list "malformed pair list"))))
       (mu:length pairs))))

;;;
;;; fixpoint loop
;;;
(mu:intern core::ns :extern "fix"
   (:lambda (fn list)
     (core:raise-unless core:functionp fn 'core:fix "not a function")
     (core:raise-unless core:listp list 'core:fix "not a list")
     (:if (core::fn-applyp fn)
          (mu:fix (:lambda (arg) (core:apply fn `(,arg))) list)
          (mu:fix fn list))))

;;;
;;; folds
;;;
(mu:intern core::ns :extern "foldl"
  (:lambda (fn init list)
      (core:raise-unless core::appliablep fn 'core:foldl "not a function")
      (core:raise-unless core:listp list 'core:foldl "not a list")
      (mu:cdr
       (mu:fix
        (:lambda (arg)
          ((:lambda (list acc)
             (:if list
                  (mu:cons
                   (mu:cdr list)
                   (core:apply fn `(,(mu:car list) ,acc)))
                  arg))
           (mu:car arg)
           (mu:cdr arg)))
        (mu:cons list init)))))

(mu:intern core::ns :extern "foldr"
  (:lambda (fn init list)
    (core:raise-unless core::appliablep fn 'core:foldr "not a function")
    (core:raise-unless core:listp list 'core:foldr "not a list")
    (core:foldl fn init (core:reverse list))))

;;;
;;; maps
;;;
(mu:intern core::ns :extern "mapc"
   (:lambda (fn list)
     (core:raise-unless core::appliablep fn 'core:mapc "not a function")
     (core:raise-unless core:listp list 'core:mapc "not a list")
     (mu:fix
      (:lambda (lst)
        (:if lst
             (core::prog2
                 (core:apply fn `(,(mu:car lst)))
                 (mu:cdr lst))
             ()))
      list)
     list))

(mu:intern core::ns :extern "mapcar"
   (:lambda (fn list)
     (core:raise-unless core::appliablep fn 'core:mapcar "not a function")
     (core:raise-unless core:listp list 'core:mapcar "not a list")
     (core:foldl
      (:lambda (elt acc)
        `(,@acc ,(core:apply fn `(,elt))))
      ()
      list)))

(mu:intern core::ns :extern "mapl"
   (:lambda (fn list)
      (core:raise-unless core::appliablep fn 'core:mapl "not a function")
      (core:raise-unless core:listp list 'core:mapl "not a list")
      (mu:fix
       (:lambda (list)
         (:if list
              (core::prog2
                  (core:apply fn `(,list))
                  (mu:cdr list))
              ()))
       list)
      list))

(mu:intern core::ns :extern "maplist"
   (:lambda (fn list)
     (core:raise-unless core::appliablep fn 'core:maplist "not a function")
     (core:raise-unless core:listp list 'core:maplist "not a list")
     (mu:car
      (mu:fix
       (:lambda (loop)
         ((:lambda (acc list)
            (:if list
                 (mu:cons
                  `(,@acc ,(core:apply fn `(,list)))
                  (mu:cdr list))
                 loop))
          (mu:car loop)
          (mu:cdr loop)))
       `(() ,@list)))))

;;;
;;; finds
;;;
(mu:intern core::ns :extern "findl"
   (:lambda (fn list)
     (core:raise-unless core::appliablep fn 'core:findl "not a function")
     (core:raise-unless core:listp list 'core:findl "not a list")
     (core:foldl
      (:lambda (el acc)
        (:if (core:apply fn `(,el))
             el
             acc))
      ()
      list)))

(mu:intern core::ns :extern "findr"
   (:lambda (fn list)
     (core:raise-unless core::appliablep fn 'core:findr "not a function")
     (core:raise-unless core:sequencep list 'core:findr "not a list")
     (core:foldr
      (:lambda (el acc)
        (:if (core:apply fn `(,el))
             el
             acc))
      ()
      list)))

;;;
;;; position
;;;
(mu:intern core::ns :extern "positionl"
   (:lambda (fn list)
     (core:raise-unless core::appliablep fn 'core:positionl "not a function")
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

(mu:intern core::ns :extern "positionr"
   (:lambda (fn list)
     (core:raise-unless core::appliablep fn 'core:positionr "not a function")
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
(mu:intern core::ns :extern "append"
   (:lambda (list)
     (core:raise-unless core:listp list 'core:append "not a list")
     (core:foldr
      (:lambda (el acc)
        (:if (core:listp el)
          (core:foldr mu:cons acc el)
          el))
      ()
      list)))

(mu:intern core::ns :extern "reverse"
  (:lambda (list)
    (core:raise-unless core:listp list 'core:reverse "not a list")
    (core:foldl mu:cons () list)))

(mu:intern core::ns :extern "dropl"
   (:lambda (list n)
      (core:raise-unless core:listp list 'core:dropl "not a list")
      (core:raise-unless core:fixnump n 'core:dropl "not a fixnum")
      (mu:nthcdr n list)))

(mu:intern core::ns :extern "dropr"
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

(mu:intern core::ns :extern "assoc"
  (:lambda (item list)
    (mu:car
     (mu:fix
      (:lambda (lst)
        (:if lst
             ((:lambda (entry)
                (core:raise-unless core:listp entry 'core:assoc "not a cons")
                (:if (core:null entry)
                     (mu:cdr lst)
                     (:if (mu:eq item (mu:car entry))
                          lst
                          (mu:cdr lst))))
              (mu:car lst))
             ()))
      list))))

(mu:intern core::ns :extern "last"
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
