(mu:intern :prelude "run-test"
   (:lambda (expr)
      ((:lambda (expansion)          
         (prelude:format :t "runtest: ~A expands to ~A => ~A~%"
                         `(,expr ,expansion ,(mu:eval expansion))))
        (mu:%qquote (mu:open :string :input expr)))))

(prelude:run-test "`1234")
(prelude:run-test "`,1234")
(prelude:run-test "`(1234)")
(prelude:run-test "`(1234 symbol)")
(prelude:run-test "`,(mu:type-of symbol)")
;;; (prelude:run-test "``1234")
;;; (prelude:run-test "`,`1234")
;;; (prelude:run-test "`,`(1234)")
(prelude:run-test "`((1234))")
(prelude:run-test "`(,1234)")
