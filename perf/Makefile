#
# perf makefile
#
.PHONY: help clean commit report run summary mu core

NTESTS ?= 20
TMPF:=$(shell mktemp)
NAMESPACES=\
	mu	\
	core

help:
	@echo "thorn perf makefile -----------------"
	@echo
	@echo "--- perf options"
	@echo "    namespaces - list namespaces"
	@echo "    list - tests in \$$NAMESPACE"
	@echo "    \$$NAMESPACE - run all tests in namespace, unformatted output"
	@echo "    base - run all tests in all namespaces, establish baseline"
	@echo "    summary - "
	@echo "    run - individual test: NAMESPACE=namespace TEST=test unformatted output"
	@echo "    report - individual test: NAMESPACE=namespace TEST=test formatted output (storage only)"
	@echo "    commit - rust tests, print perf report"
	@echo "    valgrind - run memcheck, callgrind, cachegrind reports"

valgrind: memcheck callgrind cachegrind clean

commit:
	@sed -e '1,$$s/^.[^ ]*.[ ]*//' perf.report | paste base.report - > perf.anomaly
	@python3 summary.py perf.anomaly
	@rm -f perf.repo perf.anomaly

mu:
	@for test in `cat mu/tests`; do 	\
	    bash mu/run-perf mu/$$test $(NTESTS);	\
	done

core:
	@for test in `cat core/tests`; do 	\
	    bash core/run-perf core/$$test $(NTESTS);	\
	done

namespaces:
	@echo $(NAMESPACES)

list:
	@cat $$NAMESPACE/tests

run:
	@for test in `cat $$NAMESPACE/tests`; do \
	    bash $$NAMESPACE/run-perf $$NAMESPACE/$$test;	\
	done

report:
	@bash $(NAMESPACE)/run-perf $(NAMESPACE)/$(TEST) > $(TMPF)
	@python3 storage.py $(TMPF)
	@rm -f $(TMPF)

base:
	@rm -f $(TMPF)
	@for namespace in $(NAMESPACES); do	\
	    make $$namespace >> $(TMPF);	\
	done
	@python3 report.py $(TMPF) > base.report

summary:
	@rm -f $(TMPF)
	@for namespace in $(NAMESPACES); do	\
	    make $$namespace >> $(TMPF);	\
	done
	@python3 report.py $(TMPF) > perf.report

memcheck:
	@valgrind --tool=memcheck --leak-check=yes --show-reachable=yes --num-callers=10 \
		--leak-resolution=med --show-leak-kinds=definite \
		../dist/runtime -p      \
		    -q "(mu:eval ()) " > memcheck.report 2>&1 || true

callgrind:
	@valgrind --tool=callgrind     	\
		../dist/runtime -p      \
		    -q "(mu:eval ()) " > callgrind.report 2>&1 || true

	@callgrind_annotate --auto=yes callgrind.out.* >> callgrind.report

cachegrind:
	@valgrind --tool=cachegrind	\
		../dist/runtime -p      \
		    -q "(mu:eval ()) " > cachegrind.report 2>&1 || true

clean:
	@rm -rf *.diff *.out *.out.* ../dist/runtime.dSYM
