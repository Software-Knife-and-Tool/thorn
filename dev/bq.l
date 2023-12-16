;;;
;;; backquote machine
;;;
(mu:intern :prelude "%%bq-nil" (:lambda (state next list)))

(mu:intern :prelude "%%bq-syntax"
   (:lambda (next list)
     (prelude:raise
      (mu:cdr next)
      'prelude:read "backquote syntax error")))

(mu:intern :prelude "%%bq-self"
   (:lambda (next list)
     (mu:cdr next)))

(mu:intern :prelude "%%bq-quote"
   (:lambda (next list)
     `(:quote ,(mu:car next))))

(mu:intern :prelude "%%bq-list"
   (:lambda (next list)
      `(mu:%append ,list ,(mu:cdr next))))       

(mu:intern :prelude "%bq-states"
   `((:bquote
      (:at prelude:%%bq-syntax :exit)
      (:bquote prelude:%%bq-nil :bquote)
      (:comma prelude:%%bq-nil :comma)
      (:const prelude:%%bq-self :exit)
      (:dot prelude:%%bq-syntax :exit)
      (:list prelude:%%bq-list :list)
      (:elist prelude:%%bq-list :exit)
      (:symbol prelude:%%bq-quote :exit))
     (:comma
      (:at prelude:%%bq-syntax :exit)
      (:bquote prelude:%%bq-nil :bquote)
      (:comma prelude:%%bq-nil :comma)
      (:const prelude:%%bq-self :exit)
      (:dot prelude:%%bq-syntax :exit)
      (:list prelude:%%bq-list :comma)
      (:elist prelude:%%bq-list :exit)
      (:symbol prelude:%%bq-quote :comma))
     (:list
      (:at prelude:%%bq-syntax :exit)
      (:bquote prelude:%%bq-nil :bquote)
      (:comma prelude:%%bq-nil :comma)
      (:const prelude:%%bq-self :exit)
      (:dot prelude:%%bq-syntax :exit)
      (:list prelude:%%bq-list :comma)
      (:elist prelude:%%bq-list :exit)
      (:symbol prelude:%%bq-quote :comma))))

(mu:intern :prelude "%bq-next"
   (:lambda (stream)
      ((:lambda (ch)
          (:if (prelude:null ch)
               '(:exit)
               ((:lambda (ctrl-map)
                  (:if (prelude:null ctrl-map)
                       ((:lambda (expr)
                          (:if (prelude:constantp expr)
                               (mu:cons expr :const)
                               (mu:cons expr :symbol)))
                        ((:lambda ()
                           (mu:un-char ch stream)
                           (prelude:read stream :t ()))))
                       ctrl-map))
                (prelude:%assoc
                 ch
                 `(,(mu:cons #\, :comma)
                   ,(mu:cons #\` :bquote)
                   ,(mu:cons #\@ :at)
                   ,(mu:cons #\( :list)
                   ,(mu:cons #\) :elist))))))
          (mu:rd-char stream () ()))))

(mu:intern :prelude "%bq-state-machine"
   (:lambda (stream)
     (mu:fix
      (:lambda (loop)
        ((:lambda (state next list)
           (:if (mu:eq :exit state)
                loop
                ((:lambda (token type)
                   ((:lambda (current-state)
                       (:if (prelude:null current-state)
                            (prelude:raise state 'prelude:%bq-state-machine "state machine state botch")
                            ((:lambda (current-event)
                                (:if (prelude:null current-event)
                                     (prelude:raise type 'prelude:%bq-state-machine "state machine event botch")
                                     ((:lambda (action next-state)
                                         (mu:cons next-state (mu:apply action `(,next ,list))))
                                      (mu:sy-val (mu:nth 1 current-event))
                                      (mu:nth 2 current-event))))
                             (prelude:%assoc type (mu:cdr current-state)))))
                    (prelude:%assoc state prelude:%bq-states)))
                 (mu:car next)
                 (mu:cdr next))))
           (mu:car loop)
           (prelude:%bq-next stream)
           (mu:cdr loop)))
      `(:bquote))))

(mu:intern :prelude "%bq-parse-stream"
   (:lambda (stream)
     (prelude:reverse (prelude:%bq-state-machine stream))))

(mu:intern :prelude "bq-parse-string"
   (:lambda (source)
     (:if (prelude:stringp source)
          (prelude:%bq-parse-stream (mu:open :string :input source))
          (prelude:error "bq-parse-string: ~A is not a string~%" `(source)))))
