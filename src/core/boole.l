
;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; boolean operations
;;;
(mu:intern :core "lognot"
   (:lambda (integer)
     (mu:cdr
      (mu:fix
       (:lambda (loop)
         ((:lambda (index complement)
             (:if (mu:fx-lt index 56)
                  (mu:cons (core:1+ index)
                           (mu:logor
                            complement
                            (:if (core:zerop (mu:logand integer (mu:fx-ash 1 index)))
                                 (mu:fx-ash 1 index)
                                 0)))
                  loop))
             (mu:car loop)
             (mu:cdr loop)))
         (mu:cons 0 0)))))

#|
(mu:intern :core "boole"
   (:lambda (op integer-1 integer-2)
     ((:lambda (op-table) {
        (:if (core:keywordp op)
             (:if (core:fixnump integer-1)
                  (:if (core:fixnump integer-2)
                       one
                       (core:raise integer-2 'core:boole "not a fixnum"))
                  (core:raise integer-2 'core:boole "not a fixnum"))
             (core:raise list 'core:boole "not a boole keyword")))
      `(,(mu:cons :1 (:lambda () integer-1))                                   
        ,(mu:cons :2 (:lambda () integer-2))                                   
        ,(mu:cons :andc1  and complement of integer-1 with integer-2  
        ,(mu:cons :andc2  and integer-1 with complement of integer-2  
        ,(mu:cons :and    and                                         
        ,(mu:cons :c1     complement of integer-1                     
        ,(mu:cons :c2     complement of integer-2                     
        ,(mu:cons :clr    (:lambda () 0))                    
        ,(mu:cons :eqv    equivalence (exclusive nor)                 
        ,(mu:cons :ior    inclusive or                                
        ,(mu:cons :nand   not-and                                     
        ,(mu:cons :nor    not-or                                      
        ,(mu:cons :orc1   or complement of integer-1 with integer-2   
        ,(mu:cons :orc2   or integer-1 with complement of integer-2   
        ,(mu:cons :set    (:lambda () -1))                   
        ,(mu:cons :xor    exclusive or                                
|#
