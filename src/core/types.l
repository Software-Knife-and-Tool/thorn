;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; type system
;;;
(mu:intern core::ns :intern "ctypes"
   (mu:make-ns "ctypes" core::ns))

(mu:intern core::ns :extern "ctypep"
   (:lambda (object)
      (:if (core:structp object)
           (mu:eq :ctype (mu:st-type object))
           ())))

(mu:intern core::ns :extern "type-of"
   (:lambda (object)
      ((:lambda (type)
          (:if (mu:eq type :struct)
               (:if (mu:eq :ctype (mu:st-type object))
                    (mu:sv-ref (mu:st-vec object) 0)
                    type)
               type))
       (mu:type-of object))))

(mu:intern core::ns :intern "ctype-slot"
   (:lambda (struct name)
      (core:raise-unless core:ctypep struct 'core:ctype-slot "not a ctype")
      (core:raise-unless core:keywordp name 'core:ctype-slot "not a slot name")
      ((:lambda (slot)
          (core:raise-if core:null slot 'core:ctype-slot "slot name not bound")
          (mu:car slot))
       (core::assoc slot-name (mu:sv-ref (mu:st-vec struct) 1)))))
