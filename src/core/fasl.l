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
      (core:%mapc
       (:lambda (byte)
          (:if (core:charp byte)
               (core:write-char byte stream)
               (:if (core:fixnump byte)
                    (core:write-byte byte stream)
                    (core:%prog2
                     (core:raise-unless
                      core:stringp
                      byte
                      'core:elf64-write-bytes
                      "untranslated type")
                     (core:%mapv
                      (:lambda (ch)
                         (core:write-char ch stream))
                      byte)))))
       bytes)))

(mu:intern core "elf64-read-bytes"
   (:lambda (nbytes stream)
      ((:lambda (list)
          (mu:sv-list :byte (mu:cdr list)))
       (mu:fix
        (:lambda (loop)
           ((:lambda (nbytes list)
               (:if (core:zerop nbytes)
                    loop
                    (mu:cons
                     (core:1- nbytes)
                     `(,@list ,@(core:read-byte stream :t ())))))
            (mu:car loop)
            (mu:cdr loop)))
        (mu:cons nbytes ())))))
         
(mu:intern core "elf64-read"
   (:lambda (type stream)
      (:if (mu:eq type :half)
           (mu:fx-add
            (core:read-byte stream t ())
            (core:ash (core:read-byte stream :t ()) 8))
           (:if (mu:eq type :word)
                (mu:fx-add
                 (core:elf64-read :half stream)
                 (core:ash (core:elf64-read :half stream) 16))
                (:if (mu:eq type :u64)
                     (mu:fx-add
                      (core:elf64-read :word stream)
                      (core:ash (core:elf64-read :word stream) 32))
                     (:if (mu:eq type :tag)
                          (mu:%repr :t (core:elf64-read-bytes 8 stream))
                          (core:raise size 'core:elf64-read "not a legal type")))))))

(mu:intern core "elf64-write"
   (:lambda (type n stream)
      (:if (mu:eq type :half)
           (core:elf64-write-bytes
            stream
            (core:logand n #xff)
            (core:logand (core:ash n -8) #xff))
           (:if (mu:eq type :word)
                (core:%prog2
                 (core:elf64-write :half n stream)
                 (core:elf64-write :half (core:ash n -16) stream))
                (:if (mu:eq type :u64)
                     (core:%prog2
                      (core:elf64-write :half n stream)
                      (core:elf64-write :half (core:ash n -32) stream))
                     (:if (mu:eq type :tag)
                          (core:elf64-write :u64 (mu:tag-of n) stream)
                          (t (core:raise size 'el64-write "not a legal type"))))))))

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

(mu:intern core "core:elf64-read-header"
   (:lambda (stream)
      (mu:with-ex
       (:lambda (ex)
          (core:exceptf
           mu:err-out
           "elf64-read-header exception: ~A on ~A by ~S from ~A~%"
           ()
           ex))
       (:lambda ()
          `(,(mu:cons "magic" (core:elf64-read :u64 elf64))
            ,(mu:cons "class" (core:read-byte elf64 :t ()))
            ,(mu:cons "data-encoding" (core:read-byte elf64 :t ()))
            ,(mu:cons "file-version" (core:read-byte elf64 t ()))
            ,(mu:cons "OS-ABI" (core:read-byte elf64 :t ()))
            ,(mu:cons "ABI-version" (core:read-byte elf64 :t ()))
            ,(mu:cons "e_object" (core:elf64-read :half elf64))
            ,(mu:cons "e_machine" (core:elf64-read :half elf64))
            ,(mu:cons "e_version" (core:elf64-read :half elf64))
            ,(mu:cons "e_entry" (core:elf64-read :half elf64))
            ,(mu:cons "e_phoff" (core:elf64-read :u64 elf64))
            ,(mu:cons "e_shoff" (core:elf64-read :u64 elf64))
            ,(mu:cons "e_flags" (core:elf64-read :word elf64))
            ,(mu:cons "e_ehsize" (core:elf64-read :half elf64))
            ,(mu:cons "e_phentsize" (core:elf64-read :half elf64))
            ,(mu:cons "e_phnum" (core:elf64-read :half elf64))
            ,(mu:cons "e_shentsize" (core:elf64-read :half elf64))
            ,(mu:cons "e_shnum" (core:elf64-read :half elf64))
            ,(mu:cons "e_shstrndx" (core:elf64-read :half elf64))
            ,(mu:cons "sh_name" (core:elf64-read :word elf64))
            ,(mu:cons "sh_type" (core:elf64-read :word elf64))
            ,(mu:cons "sh_flags" (core:elf64-read :u64 elf64))
            ,(mu:cons "sh_addr" (core:elf64-read :u64 elf64))
            ,(mu:cons "sh_offset" (core:elf64-read :u64 elf64))
            ,(mu:cons "sh_size" (core:elf64-read :u64 elf64))
            ,(mu:cons "sh_link" (core:elf64-read :word elf64))
            ,(mu:cons "sh_info" (core:elf64-read :word elf64))
            ,(mu:cons "sh_addralign" (core:elf64-read :u64 elf64))
            ,(mu:cons "sh_entsize" (core:elf64-read :u64 elf64)))))))

(mu:intern core "elf64-print-header"
  (:lambda (elf64-header)
     (core:raise-unless core:consp elf64-header 'core:elf64-print-header "not an association list")
     (core:%mapc
      (:lambda (entry)
         (core:format t "    ~A: ~A~%" `(,(mu:car entry) ,(mu:cdr entry))))
      elf64-header)))

(mu:intern core "elf64-dump"
   (:lambda (path)
      (core:raise-unless core:stringp path 'core:elf64-dump "not a pathname")
      ((:lambda (elf64)
          ((:lambda (magic)
              (core:raise-when core:null magic 'core:elf64-dump "error reading magic")
              (core:raise-unless (:lambda (magic) (mu:eq #x7f magic)) magic 'core:elf64-dump "wrong magic")
              (core:format :t "~A: elf64 file (magic #x~X ~A~A~A)~%"
                           `(path
                             magic
                             (core:read-char elf64 :t ())
                             (core:read-char elf64 :t ())
                             (core:read-char elf64 :t ())))
              (core:format :t "~A: elf64 file header~%" `(path))
              (core:elf64-print-header (core:elf64-read-header elf64))
              (core:format t "~A: body:~%" path)
              (mu:fix
               (:lambda (loop)
                  (:if (core:eof elf64)
                       loop
                       ((:lambda (tag)
                           (core:format :t "[~A]~%" `(,(core:type-of tag) ,tag))
                           (core:null loop))
                        (core:elf64-read :tag elf64))))
               ()))
             (core:read-byte elf64 () ()))
          (mu:open :file :input path)))))
