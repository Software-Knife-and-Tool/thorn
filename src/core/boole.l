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

(mu:intern :core "logxor"
   (:lambda (integer-1 integer-2)
     (mu:cdr
      (mu:fix
       (:lambda (loop)
         ((:lambda (index xor)
             (:if (mu:fx-lt index 56)
                  (mu:cons (core:1+ index)
                           (:if (mu:eq (mu:logand (mu:fx-ash 1 index) integer-1)
                                       (mu:logand (mu:fx-ash 1 index) integer-2))
                                xor
                                (mu:logor xor (mu:fx-ash 1 index))))
                  loop))
             (mu:car loop)
             (mu:cdr loop)))
         (mu:cons 0 0)))))

(mu:intern :core "boole"
   (:lambda (op integer-1 integer-2)
     ((:lambda (op-table)
        (:if (core:keywordp op)
             (:if (core:fixnump integer-1)
                  (:if (core:fixnump integer-2)
                       ((:lambda (entry)
                          (:if (core:null entry)
                               (core:raise op 'core:boole "not an op")
                               (mu:apply (mu:cdr entry) ())))
                        (core:%assoc op op-table))
                       (core:raise integer-2 'core:boole "not a fixnum"))
                  (core:raise integer-1 'core:boole "not a fixnum"))
             (core:raise list 'core:boole "not a boole keyword")))
      `(,(mu:cons :1 (:lambda () integer-1))
         ,(mu:cons :2 (:lambda () integer-2))
         ,(mu:cons :andc1 (:lambda () (mu:logand (core:lognot integer-1) integer-2)))
         ,(mu:cons :andc2 (:lambda () (mu:logand integer-1 (core:lognot integer-2))))
         ,(mu:cons :and (:lambda () (mu:logand integer-1 integer-2)))
         ,(mu:cons :c1 (:lambda () (core:lognot integer-1)))
         ,(mu:cons :c2 (:lambda () (core:lognot integer-2)))
         ,(mu:cons :clr (:lambda () 0))
         ,(mu:cons :ior (:lambda () (mu:logor integer-1 integer-2)))
         ,(mu:cons :nand (:lambda () (core:lognot (mu:logand integer-1 integer-2))))
         ,(mu:cons :nor (:lambda () (core:lognot (mu:logor integer-1 integer-2))))
         ,(mu:cons :orc1 (:lambda () (mu:logor (core:lognot integer-1) integer-2)))
         ,(mu:cons :orc2 (:lambda () (mu:logor integer-1 (core:lognot integer-2))))  
         ,(mu:cons :set (:lambda () -1))
         ,(mu:cons :eqv (:lambda () (core:lognot (core:logxor integer-1 integer-2))))
         ,(mu:cons :xor (:lambda () (core:logxor integer-1 integer-2)))))))
