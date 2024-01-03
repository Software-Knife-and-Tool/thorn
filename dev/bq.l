(mu:intern :prelude "qquote"
   (:lambda (expr)
     ((:lambda (expansion)
        (prelude:format :t "qquote: ~A expands to ~A~%"
                        `(,expr ,expansion)))
        (mu:%qquote (mu:open :string :input expr)))))
