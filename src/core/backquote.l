;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; backquote - list construction DSL
;;;
(mu:intern core :intern "read-backquote"
   (:lambda (stream ch)
      (mu:un-char #\` stream)
      (mu:read stream () ())))

(core:set-macro-character #\` core::read-backquote :t)
