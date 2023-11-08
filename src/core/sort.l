;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; list sorting
;;;
(mu:intern :core "%max-le"
   (:lambda (root list)
     ((:lambda (next)
        (:if (mu:eq next root)
             ()
             (mu:cons next (core:%max-le next list))))
      (core:%fx-max-le root list))))

(mu:intern :core "%min-le"
   (:lambda (root list)
     ((:lambda (next)
        (:if (mu:eq next root)
             ()
             (mu:cons next (core:%min-le next list))))
      (core:%fx-min-ge root list))))

(mu:intern :core "sort-list-ascending"
   (:lambda (list)
     (:if (core:listp list)
          ((:lambda (root max min)
             (:if (mu:eq root min)
                  `(,root
                    ,@(core:%min-le root (mu:cdr list))
                    ,@(core:%max-le root (mu:cdr list)))
                  (:if (mu:eq root max)
                       `(,@(core:reverse (core:%min-le root (mu:cdr list)))
                           ,@(core:reverse (core:%max-le root (mu:cdr list)))
                           ,root)
                       `(,@(core:reverse (core:%max-le root (mu:cdr list)))
                           ,root
                           ,@(core:%min-le root (mu:cdr list))))))
           (mu:car list)
           (core:%fx-list-max list)
           (core:%fx-list-min list))
          (core:raise list 'core:sort-list-ascending "not a list"))))

(mu:intern :core "sort-list-descending"
   (:lambda (list)
      (:if (core:listp list)
           (core:reverse (core:sort-list-ascending list))
           (core:raise list 'core:sort-list-descending "not a list"))))
