#
# perf makefile
#
.PHONY: help base clean commit current diff summary metrics mu core

NTESTS ?= 20
TMPF:=$(shell mktemp)
NAMESPACES=\
	mu	\
	core

VALGRIND='((:lambda (g) (mu:apply g (mu:cons g (mu:cons 101010101 (mu:cons 11011 ()))))) (:lambda (g a b) (:if (mu:eq 0 b) a (mu:apply g (mu:cons g (mu:cons b (mu:cons (mu:fx-sub a (mu:fx-mul b (mu:fx-div a b))) ()))))))) '

help:
	@echo "thorn perf makefile -----------------"
	@echo
	@echo "--- perf options"
	@echo "    namespaces - list namespaces"
	@echo "    list - tests in \$$NS"
	@echo "    \$$NS - run all tests in namespace, unformatted output"
	@echo "    base - run all tests in all namespaces, establish baseline report"
	@echo "    current - run all tests in all namespace, establish current report"
	@echo "    commit - compare current with base, promote current to base"
	@echo "    diff - compare current report with base report"
	@echo "    metrics - verbose print"
	@echo "    valgrind - run memcheck, callgrind, cachegrind reports"

valgrind: memcheck callgrind cachegrind clean

diff:
	@sed -e '1,$$s/^.[^ ]*.[ ]*//' current.report | paste base.report - > $(TMPF)
	@python3 summary.py $(TMPF)
	@rm -f $(TMPF)

commit:
	@sed -e '1,$$s/^.[^ ]*.[ ]*//' current.report | paste base.report - > $(TMPF)
	@python3 summary.py $(TMPF)
	@rm -f $(TMPF)
	@mv current.report base.report

mu:
	@python3 ./perf-ns.py mu . $(NTESTS)

core:
	@python3 ./perf-ns.py core . $(NTESTS)	

namespaces:
	@echo $(NAMESPACES)

list:
	@cat $$NS/tests

base:
	@rm -f base.report
	@for namespace in $(NAMESPACES); do			\
	    make -s $$namespace > $(TMPF);			\
	    python3 report-ns.py $(TMPF) >> base.report;	\
	done
	@rm -f $(TMPF)

current:
	@rm -f current.report
	@for namespace in $(NAMESPACES); do			\
	    make -s $$namespace > $(TMPF);			\
	    python3 report-ns.py $(TMPF) >> current.report;	\
	done
	@rm -f $(TMPF)

metrics:
	@for namespace in $(NAMESPACES); do			\
	    make -s $$namespace > $(TMPF);			\
	    python3 metrics-ns.py $(TMPF);			\
	done
	@rm -f $(TMPF)

memcheck:
	@valgrind --tool=memcheck --leak-check=yes --show-reachable=yes --num-callers=10 \
		--leak-resolution=med --show-leak-kinds=definite \
		../dist/runtime -p      \
		    -q $(VALGRIND) > memcheck.report 2>&1 || true

callgrind:
	@valgrind --tool=callgrind     	\
		../dist/runtime -p      \
		    -q $(VALGRIND) > callgrind.report 2>&1 || true

	@callgrind_annotate --auto=yes callgrind.out.* >> callgrind.report

cachegrind:
	@valgrind --tool=cachegrind	\
		../dist/runtime -p      \
		    -q $(VALGRIND) > cachegrind.report 2>&1 || true

clean:
