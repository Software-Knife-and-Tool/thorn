;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; types
;;;
(mu:intern :core "type-of"
   (:lambda (value)
      ((:lambda (type)
          (:if (mu:eq type :struct)
               (:if (mu:eq :<ctype> (mu:st-type value))
                    (mu:untern () (mu:sv-ref (mu:st-vec value) 0))
                    (mu:st-type value))
               (:if (core:stringp value)
                    'string
                    (mu:untern () (mu:sy-name type)))))
       (mu:type-of value))))

;;;
;;; typep
;;;
(mu:intern :core "%typespec-map%"
   `((asyncid :asyncid)
     (char (satisfies core:charp))
     (function (satisfies core:functionp))
     (map :map)
     (number (or (satisfies core:floatp) integer))
     (integer (satisfies core:fixnump))
     (sequence (or (satisfies core:vectorp) (satisfies core:stringp) (satisfies core:listp)))
     (stream (satisfies core:streamp))
     (string (satisfies core:stringp))
     (struct (satisifes core:structp))
     (symbol (or (satisfies core:symbolp) (satisfies core:keywordp)))))

(mu:intern :core "%compound-typespec-map%"
   `(,(mu:cons 'satisfies (:lambda (value spec-list)
                             (:if (mu:eq 1 (mu:length spec-list))
                                  ((:lambda (predicate-symbol)
                                      (:if (core:symbolp predicate-symbol)
                                           (:if (mu:boundp predicate-symbol)
                                                (core:apply (mu:sy-val predicate-symbol) (mu:cons value ()))
                                                (core:raise predicate 'core:typep "satisfies typespec format"))
                                           (core:raise predicate 'core:typep "satisfies typespec format")))
                                   (mu:car spec-list))
                                  (core:raise spec-list 'core:typep "satisfies typespec format"))))
     ,(mu:cons 'not (:lambda (value spec-list)
                       (:if (mu:eq 1 (mu:length spec-list))
                            (core:not (core:typep value (mu:car spec-list)))
                            (core:raise spec-list 'core:typep "no typespec format"))))
     ,(mu:cons 'and (:lambda (value spec-list)
                       (:if (core:null spec-list)
                            :t
                            (core:foldl
                             (:lambda (elt acc)
                                (core:%andf acc (core:typep value elt)))
                             :t
                             spec-list))))
     ,(mu:cons 'or (:lambda (value spec-list)
                     (:if (core:null spec-list)
                          ()
                          (core:foldl
                           (:lambda (elt acc)
                             (core:%andf acc (core:typep value elt)))
                           :t
                           spec-list))))))

(mu:intern :core "typep"
   (:lambda (value typespec)
     (:if (core:keywordp typespec)
          (mu:eq typespec (mu:type-of value))
          (:if (core:keywordp typespec)
               (mu:eq typespec (core:type-of value))
               (:if (core:symbolp typespec)
                    ((:lambda (spec-phrase)
                       (:if (core:null spec-phrase)
                            (core:raise typespec 'core:typep "typespec format")
                            (core:typep value (mu:car (mu:cdr spec-phrase)))))
                     (core:%assoc typespec core:%typespec-map%))
                    (:if (core:consp typespec)
                         ((:lambda (spec spec-list)
                            ((:lambda (spec-phrase)
                               (:if (core:null spec-phrase)
                                    (core:raise typespec 'core:typep "typespec not found")
                                    (mu:apply (mu:cdr spec-phrase) `(,value ,spec-list))))
                             (core:%assoc spec core:%compound-typespec-map%)))
                          (mu:car typespec)
                          (mu:cdr typespec))
                         (core:raise typespec 'core:typep "typespec format")))))))
