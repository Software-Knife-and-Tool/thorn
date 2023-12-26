(mu:intern :prelude "run-test"
   (:lambda (expr)
      (prelude:format :t "runtest: ~A~%" `(,expr))          
      (mu:%qquote (mu:open :string :input expr))))

(prelude:run-test "`1234")
(prelude:run-test "`(1234)")
