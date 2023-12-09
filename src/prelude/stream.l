;;;  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
;;;  SPDX-License-Identifier: MIT

;;;
;;; streams
;;;
(mu:intern :prelude "streamp" (:lambda (t) (mu:eq :stream (mu:type-of t))))

(mu:intern :prelude "eof"
  (:lambda (designator)
    (mu:eof (prelude:%read-stream-designator designator))))

;;;
;;; utilities
;;;
(mu:intern :prelude "%write-stream-designator"
  (:lambda (designator)
    (:if (prelude:null designator)
         mu:std-out
         (:if (prelude:streamp designator)
              designator
              (prelude:raise designator
                          'prelude:%write-stream-designator
                          "not a stream designator")))))

(mu:intern :prelude "%read-stream-designator"
  (:lambda (designator)
    (:if (prelude:null designator)
         mu:std-in
         (:if (mu:eq :t designator)
              mu:std-in
              (:if (prelude:streamp designator)
                   designator
                   (prelude:raise designator
                               'prelude:%read-stream-designator
                               "not a stream designator"))))))

;;;
;;; constructors
;;;
(mu:intern :prelude "make-string-stream"
   (:lambda (dir init)
      (:if (prelude:keywordp dir)
           (:if (prelude:stringp init)
                (:if (mu:eq dir :input)
                     (mu:open :string :input init)
                     (:if (mu:eq dir :output)
                          (mu:open :string :output init)
                          (prelude:raise dir 'prelude:make-string-stream "not a direction keyword")))
                (prelude:raise dir 'prelude:make-string-stream "not a direction keyword"))
           (prelude:raise init 'prelude:make-string-stream "not a string initializer"))))

(mu:intern :prelude "open"
  (:lambda (dir path)
     (:if (prelude:stringp path)
          (:if (prelude:keywordp dir)
               (:if (mu:eq dir :input)
                    (mu:open :file :input path)
                    (:if (mu:eq dir :output)
                         (mu:open :file :output path)
                         (prelude:raise dir 'prelude:open "not a direction keyword")))
               (prelude:raise path 'prelude:open "not a path string"))
          (prelude:raise dir 'prelude:open "not a direction keyword"))))

(mu:intern :prelude "close"
   (:lambda (stream)
      (:if (prelude:streamp stream)
           (mu:close stream)
           (prelude:raise stream 'prelude:close "not a stream"))))

(mu:intern :prelude "with-open-stream"
   (:lambda (type dir specifier fn)
     (:if (prelude:functionp fn)
          (:if (prelude:stringp specifier)
               (:if (prelude:%orf (mu:eq :file type)
                               (mu:eq :string type))
                    (:if (prelude:%orf (mu:eq :input dir)
                                    (mu:eq :output dir))
                         ((:lambda (stream)
                             (mu:with-ex
                              (:lambda (ex) (mu:close stream) ex)
                              (:lambda () (prelude:apply fn (mu:cons stream ())))))
                          (mu:open type dir specifier))
                         (prelude:raise dir 'prelude:with-open-stream "not a direction keyword"))
                    (prelude:raise type 'prelude:with-open-stream "not a type keyword"))
               (prelude:raise path 'prelude:with-open-stream "not a stream specifier"))
          (prelude:raise dir 'prelude:with-open-stream "not a function"))))

;;;
;;; chars, bytes, and unread
;;;
(mu:intern :prelude "write-char"
   (:lambda (char designator)
     (mu:wr-char char (prelude:%write-stream-designator designator))))

(mu:intern :prelude "write-byte"
   (:lambda (byte designator)
     (mu:wr-byte byte (prelude:%write-stream-designator designator))))

(mu:intern :prelude "read-char"
  (:lambda (designator error-eofp eof-value)
    ((:lambda (stream)
       ((:lambda (ch)
          (:if (prelude:null ch)
               (:if error-eofp
                    (prelude:raise stream 'prelude:read-char "end of file")
                    eof-value)
               ch))
        (mu:rd-char stream () ())))
     (prelude:%read-stream-designator designator))))

(mu:intern :prelude "read-byte"
   (:lambda (designator error-eofp eof-value)
     ((:lambda (stream)
        (:if (prelude:eof stream)
             (:if error-eofp
                  (prelude:raise stream 'prelude:read-byte "end of file")
                  eof-value)
             (mu:rd-byte stream () ())))
      (prelude:%read-stream-designator designator))))

(mu:intern :prelude "unread-char"
   (:lambda (ch designator)
      (:if (prelude:charp ch)
           (mu:un-char ch (prelude:%write-stream-designator designator))
           (prelude:raise ch 'prelude:unread-char "not a char"))))

;;;
;;; read/write
;;;
(mu:intern :prelude "read"
   (:lambda (designator eof-error eof-value)
     ((:lambda (stream)
        (:if (prelude:eof stream)
             (:if eof-error
                  ((:lambda ()
                     (prelude:raise stream 'prelude:read "unexpected end of file")
                     eof-value))
                  ())
             ((:lambda (form)
                (:if (mu:eq form prelude:%read-list-eol%)
                     (prelude:raise stream 'prelude:read "unmatched close parenthesis")
                     form))
              (prelude:%read stream))))
      (prelude:%read-stream-designator designator))))

(mu:intern :prelude "write"
   (:lambda (value escape designator)
      ((:lambda (stream)
          (mu:write value escape stream))  
      (prelude:%read-stream-designator designator))))
