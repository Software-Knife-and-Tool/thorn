#
# thorn makefile
#
.PHONY: thorn core
SRC = ../src

# order dependent
CORE = \
	core.l		\
	closures.l	\
	fixnums.l	\
	read-macro.l	\
	read.l		\
	sequences.l	\
	symbol-macro.l	\
	symbols.l	\
	vectors.l	\
        compile.l	\
        exceptions.l	\
        format.l	\
        lambda.l	\
        lists.l		\
        load.l		\
        macro.l		\
        parse.l		\
        perf.l		\
        backquote.l	\
        streams.l	\
        strings.l	\
	types.l

PREFACE = \
	preface.l   	\
	common.l   	\
	compile.l	\
	describe.l 	\
	elf64.l		\
	lists.l	    	\
	metrics.l	\
	repl.l	    	\
	require.l   	\
	state.l	    	\
        environment.l	\
        print.l

thorn:
	@cp -r $(SRC)/core thorn/thorn
	@cp -r $(SRC)/preface thorn/thorn
	@cp -r $(SRC)/mu thorn/thorn

core:
	@rm -f core.l
	@for core in $(CORE); do		\
	    cat $(SRC)/core/$$core >> core.l;	\
	done
