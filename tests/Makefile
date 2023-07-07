#
# tests makefile
#
.PHONY: help cargo clean commit summary mu core

TMPF:=$(shell mktemp)
NAMESPACES=\
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
	@echo "    summary - run all tests in all namespaces and print summary"
cargo:
	@cargo test | grep "test result"

mu:
	@for test in `cat mu/tests`; do 		\
	    python3 mu/runner.py mu/$$test > foo;	\
	    python3 test.py mu $$test foo;		\
	done
	@rm -f foo

core:
	@for test in `cat core/tests`; do 		\
	    python3 core/runner.py core/$$test > foo;	\
	    python3 test.py core $$test foo;		\
	done
	@rm -f foo

namespaces:
	@echo $(NAMESPACES)

list:
	@cat $$NAMESPACE/tests

commit:
	@make summary --no-print-directory > tests.summary
	@grep -v "failed: 0        aborted: 0" tests.summary | grep -v -- "----"
	@git diff --unified=0 tests.summary

summary:
	@for namespace in $(NAMESPACES); do			\
	    make $$namespace --no-print-directory > out;	\
	    python3 summary.py out;				\
	done
	@rm -f out

clean:
	@rm -f *.diff *.out
