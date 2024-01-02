(mu:intern :prelude "qquote"
   (:lambda (expr)
     ((:lambda (expansion)
        ((:lambda (value)
           (prelude:format :t "qquote: ~A expands to ~A => ~A~%"
                           `(,expr ,expansion ,value))
           value)
         (mu:eval expansion)))
        (mu:%qquote (mu:open :string :input expr)))))
