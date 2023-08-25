;;;  SPDX-FileCopyrightText: Copyright 2017-2022 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; fasl utilities
;;;

;;;
;;; loader
;;;
(mu:intern core "load"
  (:lambda (source verbose)
    (core:raise-unless core:stringp source 'core:load "not a file designator")
    ((:lambda (ifs)
       (core:raise-unless core:streamp ifs 'core:load "cannot open input file")
       (:if verbose
            (core:format :t ";;; loading ~A~%" `(,source))
            ())
       (mu:fix
        (:lambda (loop)
          (:if (core:eof ifs)
               loop
               (core:with-exception
                (:lambda (exception)
                     (core:exceptf mu:err-out "load exception: ~A~%" () exception))
                (:lambda ()
                   ((:lambda (form)
                      (:if (core:eof ifs)
                           loop
                           (:if verbose
                                (core:format :t "~A~%" `(,form))
                                ()))
                      (core:null loop))
                    (mu:eval (core:compile (core:read ifs :t ()))))))))
        ()))
     (mu:open :file :input source))))

;;;
;;; file compiler
;;;
(mu:intern core "compile-file"
  (:lambda (source dest verbose)
    (core:raise-unless core:stringp source 'core:compile-file "not a file designator")
    ((:lambda (ifs ofs)
       (core:raise-unless core:streamp ifs 'core:compile-file "cannot open input file")
       (core:raise-unless core:streamp ofs 'core:compile-file "cannot open output file")
       (:if verbose
            (core:format :t ";;; compiling ~A~%" `(,source))
            ())
       (core:format ofs "(core:%load-fasl ~S #(:t ~%" `(source)) 
       (mu:fix
        (:lambda (loop)
          (:if (core:eof ifs)
               loop
               (core:with-exception
                (:lambda (ex)
                   (core:exceptf mu:err-out "compile exception: ~A on ~A by ~S from ~A~%" () ex)
                   loop)
                (:lambda ()
                   ((:lambda (form)
                       (:if (core:eof ifs)
                           loop
                           ((:lambda ()
                               (:if verbose
                                    (core:format :t "form: ~S~%" `(,form))
                                    ())
                               (core:format ofs " ~S~%" `(,(mu:compile form)))
                               (core:null loop)))))
                    (core:read ifs :t ()))))))
        ())
       (core:format ofs ")" ())
       (mu:close ifs)
       (mu:close ofs))
     (mu:open :file :input source)
     (mu:open :file :output dest))))

;;;
;;; elf64 utilities
;;;
(mu:intern core "elf64-write-bytes"
   (:lambda (stream bytes)
      (core:mapc
       (:lambda (byte)
          (:if (core:charp byte)
               (core:write-char byte stream)
               (:if (core:fixnump byte)
                    (core:write-byte byte stream)
                    (:if (core:stringp byte)
                         (core:%mapv
                          (:lambda (ch)
                             (core:write-char ch stream))
                          byte)
                         (core:raise byte 'core:elf64-write-bytes "untranslated type")))))
       bytes)))

#|
(mu:intern core "elf64-read"
   (:lambda (type elf64)
  (let ((elf64-read (lambda (type stream) (funcall core:lf64-read type stream))))
    (cond
      ((eq type :half)
       (mu:fx-add (read-byte elf64 t ()) (core:ash (read-byte elf64 :t ()) 8)))
      ((eq type :word)
       (fixnum+ (elf64-read :half elf64) (core:ash (elf64-read :half elf64) 16)))
      ((eq type :u64)
       (fixnum+ (elf64-read :word elf64) (core:ash (elf64-read :word elf64) 32)))
      ((eq type :tag) (mu:of-tag (elf64-read :u64 elf64)))
      (t (error size "elf64-read: not a legal type")))))

(mu:intern core elf64-write (type n stream)
  (let ((core:elf64-write (lambda (type n stream) (funcall preface:core:elf64-write type n stream))))
    (cond
      ((mu:eq type :half)
       (core:elf64-write-bytes stream
                    (core:logand n #xff)
                    (core:logand (core:ash n -8) #xff)))
      ((mu:eq type :word)
       (core:elf64-write :half n stream)
       (core:elf64-write :half (core:ash n -16) stream))
      ((mu:eq type :u64)
       (core:elf64-write :half n stream)
       (core:elf64-write :half (core:ash n -32) stream))
      ((mu:eq type :tag)
       (core:elf64-write :u64 (mu:tag-of n) stream))
      (t (core:raise size 'el64-write "not a legal type")))))
|#

(mu:intern core "elf64-write-header"
   (:lambda (out size)
      (core:raise-unless core:streamp out 'elf64-write-header "not a stream designator")
      ;;; header 64 bit little endian current version
      (core:elf64-write-bytes `(,out #x7f "ELF" 2 1 1 0 0 0 0 0 0 0 0 0))
                                         ; 0 e_ident
      (core:elf64-write :half 0 out)     ; 16 e_type
      (core:elf64-write :half 0 out)     ; 18 e_machine
      (core:elf64-write :half 1 out)     ; 20 e_version
      (core:elf64-write :half 0 out)     ; 22 e_entry
      (core:elf64-write :u64 0 out)      ; 24 e_phoff
      (core:elf64-write :u64 56 out)     ; 32 e_shoff
      (core:elf64-write :word 0 out)     ; 40 e_flags
      (core:elf64-write :half 0 out)     ; 42 e_ehsize
      (core:elf64-write :half 0 out)     ; 44 e_phentsize
      (core:elf64-write :half 0 out)     ; 46 e_phnum
      (core:elf64-write :half 0 out)     ; 48 e_shentsize
      (core:elf64-write :half 0 out)     ; 50 e_shnum
      (core:elf64-write :half 0 out)     ; 52 e_shstrndx
      (core:elf64-write :word 0 out)     ; 56 sh_name
      (core:elf64-write :word 0 out)     ; 60 sh_type
      (core:elf64-write :u64 0 out)      ; 68 sh_flags
      (core:elf64-write :u64 0 out)      ; 76 sh_addr
      (core:elf64-write :u64 0 out)      ; 84 sh_offset
      (core:elf64-write :u64 size out)   ; 92 sh_size
      (core:elf64-write :word 0 out)     ; 100 sh_link
      (core:elf64-write :word 0 out)     ; 102 sh_info
      (core:elf64-write :u64 0 out)      ; 110 sh_addralign
      (core:elf64-write :u64 0 out)))    ; 118 sh_entsize

#|
(mu:intern core "core:elf64-read-header" (elf64)
  (mu:with-ex
      (lambda (ex)
        (format t "error: exception while reading header~%")
        (print-except ex "read-header")
        ())
    (lambda ()
      (let ((top
             (list
              (cons "magic" (core:elf64-read :u64 elf64))
              (cons "class" (read-byte elf64 t ()))
              (cons "data-encoding" (read-byte elf64 t ()))
              (cons "file-version" (read-byte elf64 t ()))
              (cons "OS-ABI" (read-byte elf64 t ()))
              (cons "ABI-version" (read-byte elf64 t ())))))
        (dotimes (i 6) (read-byte elf64 t ()))
        (append top
                (list
                 (cons "e_object" (core:elf64-read :half elf64))
                 (cons "e_machine" (core:elf64-read :half elf64))
                 (cons "e_version" (core:elf64-read :half elf64))
                 (cons "e_entry" (core:elf64-read :half elf64))
                 (cons "e_phoff" (core:elf64-read :u64 elf64))
                 (cons "e_shoff" (core:elf64-read :u64 elf64))
                 (cons "e_flags" (core:elf64-read :word elf64))
                 (cons "e_ehsize" (core:elf64-read :half elf64))
                 (cons "e_phentsize" (core:elf64-read :half elf64))
                 (cons "e_phnum" (core:elf64-read :half elf64))
                 (cons "e_shentsize" (core:elf64-read :half elf64))
                 (cons "e_shnum" (core:elf64-read :half elf64))
                 (cons "e_shstrndx" (core:elf64-read :half elf64))
                 (cons "sh_name" (core:elf64-read :word elf64))
                 (cons "sh_type" (core:elf64-read :word elf64))
                 (cons "sh_flags" (core:elf64-read :u64 elf64))
                 (cons "sh_addr" (core:elf64-read :u64 elf64))
                 (cons "sh_offset" (core:elf64-read :u64 elf64))
                 (cons "sh_size" (core:elf64-read :u64 elf64))
                 (cons "sh_link" (core:elf64-read :word elf64))
                 (cons "sh_info" (core:elf64-read :word elf64))
                 (cons "sh_addralign" (core:elf64-read :u64 elf64))
                 (cons "sh_entsize" (core:elf64-read :u64 elf64))))))))

(mu:intern core core:elf64-print-header (core:elf64-header)
  (errorp-unless consp core:elf64-header "print-header: not an association list")
  (mapc
   (lambda (entry)
     (format t "    ~A: ~A~%" (car entry) (cdr entry)))
   core:elf64-header))

(mu:intern core core:elf64-dump (path)
  (errorp-unless stringp path "core:elf64-dump: not a pathname")
  (with-open-file elf64 path :input
     (let ((magic (read-byte elf64 () ())))
       (if (or (null magic) (eofp elf64) (not (eq #x7f magic)))
           (format t "~A: not a elf64 file, magic #x~X~%" path magic)            
           (progn
             (format t "~A: elf64 file (magic #x~X ~A~A~A)~%" path magic
                     (ansi-ch (read-byte elf64 t ()))
                     (ansi-ch (read-byte elf64 t ()))
                     (ansi-ch (read-byte elf64 t ())))
             (format t "~A: elf64 file header~%" path)
             (core:elf64-print-header (core:elf64-read-header elf64))
             (format t "~A: body:~%" path)
             (do* ((tag (core:elf64-read :tag elf64) (core:elf64-read :tag elf64)))
                  ((eofp elf64) t)
               (format t "[~A]~%" (type-of tag) tag)))))))
|#
