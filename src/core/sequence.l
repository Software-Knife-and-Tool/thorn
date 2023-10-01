;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; sequences
;;;
(mu:intern :core "sequencep" (:lambda (t) (:if (core:listp t) :t (core:vectorp t))))

(mu:intern :core "length"
   (:lambda (seq)
      (:if (core:sequencep seq)
           (:if (core:listp seq)
                (mu:length seq)
                (mu:sv-len seq))
           (core:raise seq 'core:length "not a sequence"))))

(mu:intern :core "elt"
   (:lambda (seq nth)
      (:if (core:sequencep seq)
           (:if (core:fixnump nth)
                (:if (core:listp seq)
                     (mu:nth nth seq)
                     (mu:sv-ref seq nth))
                (core:raise seq 'core:elt "not a sequence"))
           (core:raise nth 'core:elt "not a fixnum"))))
