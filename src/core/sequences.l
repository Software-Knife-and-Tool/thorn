;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; sequences
;;;
(mu:intern core "sequencep" (:lambda (t) (:if (core:listp t) :t (core:vectorp t))))

(mu:intern core "length"
   (:lambda (seq)
      (core:raise-unless core:sequencep seq 'core:length "not a sequence")
      (:if (core:listp seq)
           (mu:length seq)
           (mu:sv-len seq))))

(mu:intern core "elt"
   (:lambda (seq nth)
      (core:raise-unless core:sequencep seq 'core:elt "not a sequence")
      (core:raise-unless core:fixnump nth 'core:elt "not a fixnum")
      (:if (core:listp seq)
           (mu:nth nth seq)
           (mu:sv-ref seq nth))))
