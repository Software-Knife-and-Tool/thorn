;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; core streams
;;;
(mu:intern :core "streamp" (:lambda (t) (mu:eq :stream (mu:type-of t))))

(mu:intern :core "eof"
  (:lambda (designator)
    (mu:eof (core:%read-stream-designator designator))))

;;;
;;; utilities
;;;
(mu:intern :core "%write-stream-designator"
  (:lambda (designator)
    (:if (core:null designator)
         mu:std-out
         (:if (core:streamp designator)
              designator
              (core:raise designator
                          'core:%write-stream-designator
                          "not a stream designator")))))

(mu:intern :core "%read-stream-designator"
  (:lambda (designator)
    (:if (core:null designator)
         mu:std-in
         (:if (mu:eq :t designator)
              mu:std-in
              (:if (core:streamp designator)
                   designator
                   (core:raise designator
                               'core:%read-stream-designator
                               "not a stream designator"))))))

;;;
;;; constructors
;;;
(mu:intern :core "make-string-stream"
   (:lambda (dir init)
      (:if (core:keywordp dir)
           (:if (core:stringp init)
                (:if (mu:eq dir :input)
                     (mu:open :string :input init)
                     (:if (mu:eq dir :output)
                          (mu:open :string :output init)
                          (core:raise dir 'core:make-string-stream "not a direction keyword")))
                (core:raise dir 'core:make-string-stream "not a direction keyword"))
           (core:raise init 'core:make-string-stream "not a string initializer"))))

(mu:intern :core "open"
  (:lambda (dir path)
     (:if (core:stringp path)
          (:if (core:keywordp dir)
               (:if (mu:eq dir :input)
                    (mu:open :file :input path)
                    (:if (mu:eq dir :output)
                         (mu:open :file :output path)
                         (core:raise dir 'core:open "not a direction keyword")))
               (core:raise path 'core:open "not a path string"))
          (core:raise dir 'core:open "not a direction keyword"))))

(mu:intern :core "close"
   (:lambda (stream)
      (:if (core:streamp stream)
           (mu:close stream)
           (core:raise stream 'core:close "not a stream"))))

(mu:intern :core "with-open-stream"
   (:lambda (type dir specifier fn)
     (:if (core:functionp fn)
          (:if (core:stringp specifier)
               (:if (core:%orf (mu:eq :file type)
                               (mu:eq :string type))
                    (:if (core:%orf (mu:eq :input dir)
                                    (mu:eq :output dir))
                         ((:lambda (stream)
                             (mu:with-ex
                              (:lambda (ex) (mu:close stream) ex)
                              (:lambda () (core:apply fn (mu:cons stream ())))))
                          (mu:open type dir specifier))
                         (core:raise dir 'core:with-open-stream "not a direction keyword"))
                    (core:raise type 'core:with-open-stream "not a type keyword"))
               (core:raise path 'core:with-open-stream "not a stream specifier"))
          (core:raise dir 'core:with-open-stream "not a function"))))

;;;
;;; chars, bytes, and unread
;;;
(mu:intern :core "write-char"
   (:lambda (char designator)
     (mu:wr-char char (core:%write-stream-designator designator))))

(mu:intern :core "write-byte"
   (:lambda (byte designator)
     (mu:wr-byte byte (core:%write-stream-designator designator))))

(mu:intern :core "read-char"
  (:lambda (designator error-eofp eof-value)
    ((:lambda (stream)
       ((:lambda (ch)
          (:if (core:null ch)
               (:if error-eofp
                    (core:raise stream 'core:read-char "end of file")
                    eof-value)
               ch))
        (mu:rd-char stream () ())))
     (core:%read-stream-designator designator))))

(mu:intern :core "read-byte"
   (:lambda (designator error-eofp eof-value)
     ((:lambda (stream)
        (:if (core:eof stream)
             (:if error-eofp
                  (core:raise stream 'core:read-byte "end of file")
                  eof-value)
             (mu:rd-byte stream () ())))
      (core:%read-stream-designator designator))))

(mu:intern :core "unread-char"
   (:lambda (ch designator)
      (:if (core:charp ch)
           (mu:un-char ch (core:%write-stream-designator designator))
           (core:raise ch 'core:unread-char "not a char"))))

;;;
;;; read/write
;;;
(mu:intern :core "read"
   (:lambda (designator eof-error eof-value)
     ((:lambda (stream)
        (:if (core:eof stream)
             (:if eof-error
                  ((:lambda ()
                     (core:raise stream 'core:read "unexpected end of file")
                     eof-value))
                  ())
             ((:lambda (form)
                (:if (mu:eq form core:%read-list-eol%)
                     (core:raise stream 'core:read "unmatched close parenthesis")
                     form))
              (core:%read stream))))
      (core:%read-stream-designator designator))))

(mu:intern :core "write"
   (:lambda (value escape designator)
      ((:lambda (writer)
         (:if (core:null writer)
              (mu:write value escape (core:%write-stream-designator designator))
              (mu:apply (mu:cdr writer) `(,value ,escape ,(core:%write-stream-designator)))))
       (core:%assoc (core:type-of value)
        `((mu:cons "except" (:lambda (exception escape stream) (core:%write-exception exception escape stream)))
          (mu:cons "funct" (:lambda (function escape stream) (core:%write-function function escape stream)))
          (mu:cons "lambda" (:lambda (lambda escape stream) (core:%write-lambda lambda escape stream)))
          (mu:cons "namespc" (:lambda (namespace escape stream) (core:%write-namespace namespace escape stream)))
          (mu:cons "string" (:lambda (string escape stream) (core:%write-string string escape stream)))
          (mu:cons "vector" (:lambda (vector escape stream) (core:%write-vector vector escape stream))))))))

(mu:intern :core "terpri"
   (:lambda (designator)
      (core:write-char #\linefeed (core:%write-stream-designator designator))))
