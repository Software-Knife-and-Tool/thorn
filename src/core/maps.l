;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; folds and maps
;;;
(mu:intern core "foldl"
  (:lambda (fn init list)
      (core:raise-unless core:%appliablep fn 'core:foldl "not a function")
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

(mu:intern core "foldr"
  (:lambda (fn init list)
    (core:raise-unless core:%appliablep fn 'core:foldr "not a function")
    (core:raise-unless core:listp list 'core:foldr "not a list")
    (core:foldl fn init (core:reverse list))))

;;;
;;; maps
;;;
(mu:intern core "mapc"
   (:lambda (fn list)
     (core:raise-unless core:%appliablep fn 'core:mapc "not a function")
     (core:raise-unless core:listp list 'core:mapc "not a list")
     (mu:fix
      (:lambda (lst)
        (:if lst
             (core:%prog2
                 (core:apply fn `(,(mu:car lst)))
                 (mu:cdr lst))
             ()))
      list)
     list))

(mu:intern core "mapcar"
   (:lambda (fn list)
     (core:raise-unless core:%appliablep fn 'core:mapcar "not a function")
     (core:raise-unless core:listp list 'core:mapcar "not a list")
     (core:foldl
      (:lambda (elt acc)
        `(,@acc ,(core:apply fn `(,elt))))
      ()
      list)))

(mu:intern core "mapl"
   (:lambda (fn list)
      (core:raise-unless core:$appliablep fn 'core:mapl "not a function")
      (core:raise-unless core:listp list 'core:mapl "not a list")
      (mu:fix
       (:lambda (list)
         (:if list
              (core:%prog2
                  (core:apply fn `(,list))
                  (mu:cdr list))
              ()))
       list)
      list))

(mu:intern core "maplist"
   (:lambda (fn list)
     (core:raise-unless core:%appliablep fn 'core:maplist "not a function")
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

(mu:intern core "assoc"
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
