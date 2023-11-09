;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; list sorting
;;;
(mu:intern :core "%list-n-of"
   (:lambda (item n)          
     (mu:cdr          
      (mu:fix
       (:lambda (loop)
         ((:lambda (count list)
            (:if (core:zerop count)
                 loop
                 (mu:cons (core:1- count) (mu:cons item list))))
          (mu:car loop)
          (mu:cdr loop)))
       (mu:cons n ())))))

(mu:intern :core "%max-le"
   (:lambda (root list)
     ((:lambda (next)
        (:if (mu:eq next root)
             ()
             `(,@(core:%list-n-of next (core:count next list)) ,@(core:%max-le next list))))
      (core:%fx-max-le root list))))

(mu:intern :core "%min-le"
   (:lambda (root list)
     ((:lambda (next)
        (:if (mu:eq next root)
             ()
             `(,@(core:%list-n-of next (core:count next list)) ,@(core:%min-le next list))))
      (core:%fx-min-ge root list))))

(mu:intern :core "sort-list-ascending"
   (:lambda (list)
     (:if (core:listp list)
          ((:lambda (root max min)
             (:if (mu:eq root min)
                  `(,@(core:%list-n-of root (core:count root list))
                    ,@(core:%min-le root list)
                    ,@(core:%max-le root list))
                  (:if (mu:eq root max)
                       `(,@(core:reverse (core:%min-le root list))
                         ,@(core:reverse (core:%max-le root list))
                         ,@(core:%list-n-of root (core:count root list)))
                       `(,@(core:reverse (core:%max-le root list))
                         ,@(core:%list-n-of root (core:count root list))
                         ,@(core:%min-le root list)))))
           (mu:car list)
           (core:%fx-list-max list)
           (core:%fx-list-min list))
          (core:raise list 'core:sort-list-ascending "not a list"))))

(mu:intern :core "sort-list-descending"
   (:lambda (list)
      (:if (core:listp list)
           (core:reverse (core:sort-list-ascending list))
           (core:raise list 'core:sort-list-descending "not a list"))))
