;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; folds and maps
;;;
(mu:intern :core "%foldl"
  (:lambda (fn init list)
     (mu:cdr
      (mu:fix
       (:lambda (arg)
          ((:lambda (list acc)
              (:if list
                   (mu:cons
                    (mu:cdr list)
                    (mu:apply fn `(,(mu:car list) ,acc)))
                   arg))
           (mu:car arg)
           (mu:cdr arg)))
       (mu:cons list init)))))

(mu:intern :core "foldl"
  (:lambda (fn init list)
     (:if (core:functionp fn)
          (:if (core:listp list)
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
                 (mu:cons list init)))
               (core:raise list 'core:foldl "not a list"))
          (core:raise fn 'core:foldl "not a function"))))

(mu:intern :core "foldr"
   (:lambda (fn init list)
    (:if (core:functionp fn)
         (:if (core:listp list)
              (core:foldl fn init (core:reverse list))
              (core:raise list 'core:foldr "not a list"))
         (core:raise fn 'core:foldr "not a function"))))

;;;
;;; maps
;;;
(mu:intern :core "%mapc"
   (:lambda (fn list)
     (mu:fix
      (:lambda (lst)
        (:if lst
             ((:lambda ()
                 (mu:apply fn (mu:cons (mu:car lst) ()))
                 (mu:cdr lst)))
             ()))
      list)
     list))

(mu:intern :core "mapc"
   (:lambda (fn list)
     (:if (core:functionp fn)
          (:if (core:listp list)
               (mu:fix
                (:lambda (lst)
                   (:if lst
                        ((:lambda ()
                            (core:apply fn `(,(mu:car lst)))
                            (mu:cdr lst)))
                        ()))
                list)
               (core:raise list 'core:mapc "not a list"))
          (core:raise fn 'core:mapc "not a function"))
     list))

(mu:intern :core "%mapcar"
   (:lambda (fn list)
      (core:%foldl
       (:lambda (elt acc)
          `(,@acc ,(mu:apply fn `(,elt))))
       ()
       list)))

(mu:intern :core "mapcar"
   (:lambda (fn list)
     (:if (core:functionp fn)
          (:if (core:listp list)
               (core:%foldl
                (:lambda (elt acc)
                   `(,@acc ,(core:apply fn `(,elt))))
                ()
                list)
               (core:raise list 'core:mapcar "not a list"))
          (core:raise fn 'core:mapcar "not a function"))))

(mu:intern :core "mapl"
   (:lambda (fn list)
      (:if (core:functionp fn)
           (:if (core:listp list)
                (mu:fix
                 (:lambda (list)
                    (:if list
                         ((:lambda ()
                             (core:apply fn `(,list))
                             (mu:cdr list)))
                         ()))
                 list)
                (core:raise list 'core:mapl "not a list"))
           (core:raise fn 'core:mapl "not a function"))
      list))

(mu:intern :core "maplist"
   (:lambda (fn list)
      (:if (core:functionp fn)
           (:if (core:listp list)
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
                  `(() ,@list)))
                (core:raise list 'core:maplist "not a list"))
           (core:raise fn 'core:maplist "not a function"))))

(mu:intern :core "%assoc"
   (:lambda (item alist)
      (mu:car
       (mu:fix
        (:lambda (lst)
           (:if lst
                ((:lambda (entry cdr)
                    (:if entry
                         (:if (mu:eq item (mu:car entry))
                              lst
                              cdr)
                         cdr))
                 (mu:car lst)
                 (mu:cdr lst))
                ()))
        alist))))

(mu:intern :core "assoc"
   (:lambda (item alist)
      (:if (core:listp alist)
           (mu:car
            (mu:fix
             (:lambda (lst)
                (:if lst
                     ((:lambda (entry)
                         (:if (core:listp entry)
                              (:if (core:null entry)
                                   (mu:cdr lst)
                                   (:if (mu:eq item (mu:car entry))
                                        lst
                                        (mu:cdr lst)))
                              (core:raise entry 'core:assoc "not a cons")))
                      (mu:car lst))
                     ()))
             alist))
         (core:raise alist 'core:assoc "not an alist"))))
