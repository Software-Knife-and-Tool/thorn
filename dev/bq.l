(mu:intern () "qquote"
   (:lambda (expr)
     ((:lambda (expansion)
        ((:lambda (value)
           (prelude:format :t "qquote: ~A expands to ~A~%"
                           `(,expr ,expansion))
           (prelude:format :t  "=> ~A~%" `(,value))
           value)
         (mu:eval expansion)))
        (mu:%qquote (mu:open :string :input expr)))))
