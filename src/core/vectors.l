;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;;  core vectors
;;;
(mu:intern core "%mapv"
  (:lambda (fn vector)
    (mu:fix
     (:lambda (nth)
       (:if nth
            (:if (mu:eq nth (mu:sv-len vector))
                 ()
                 ((:lambda ()
                     (core:funcall fn `(,(mu:sv-ref vector nth)))
                     (mu:fx-add 1 nth))))
            ()))
     0)))
