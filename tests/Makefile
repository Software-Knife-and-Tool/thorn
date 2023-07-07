#
# tests makefile
#
.PHONY: help cargo clean commit namespaces list raw report summary mu core

TMPF:=$(shell mktemp)
NAMESPACES=	\
	mu	\
	core

help:
	@echo "thorn test makefile -----------------"
	@echo
	@echo "--- test options"
	@echo "    cargo - run rust tests"
	@echo "    namespaces - list namespaces"
	@echo "    commit - create test summary"
	@echo "    list - tests in \$$NAMESPACE"
	@echo "    \$$NAMESPACE - run all tests in namespace, raw output"
	@echo "    raw - run \$$TEST in \$$NAMESPACE, unprocessed output"
	@echo "    report - run \$$TEST in \$$NAMESPACE, report output"
	@echo "    summary - run all tests in all namespaces and print summary"

cargo:
	@cargo test | grep "test result"

namespaces:
	@echo $(NAMESPACES)

list:
	@cat $$NAMESPACE/tests

mu:
	@for test in `cat mu/tests`; do 		\
	    python3 mu/runner.py mu/$$test > $(TMPF);	\
	    python3 test.py mu $$test $(TMPF);		\
	done
	@rm -f $(TMPF)

core:
	@for test in `cat core/tests`; do 			\
	    python3 core/runner.py core/$$test > $(TMPF);	\
	    python3 test.py core $$test $(TMPF);		\
	done
	@rm -f $(TMPF)

commit:
	@make summary --no-print-directory > tests.summary
	@echo "Test Summaries"
	@echo -n "--------------"
	@grep -v "failed: 0        aborted: 0" tests.summary | grep -v -- "----" | grep -v "Test Summary"
	@echo "Test Diffs"
	@echo "-------------"
	@git diff --unified=0 tests.summary

summary:
	@for namespace in $(NAMESPACES); do			\
	    make $$namespace --no-print-directory > $(TMPF);	\
	    python3 summary.py $(TMPF);				\
	done
	@rm -f $(TMPF)

raw:
	@python3 $(NAMESPACE)/runner.py $(NAMESPACE)/$(TEST)

report:
	@python3 $(NAMESPACE)/runner.py $(NAMESPACE)/$(TEST) > $(TMPF)
	@python3 report.py $(NAMESPACE) $(TEST) $(TMPF)
	@rm -f $(TMPF)

clean:
	@rm -f *.diff *.out
