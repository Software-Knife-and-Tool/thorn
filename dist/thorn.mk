#
# thorn makefile
#
.PHONY: thorn core
SRC = ../src

# core.l needs to be first
CORE = \
	core.l			\
	closures.l		\
	compile.l		\
	describe.l		\
	exceptions.l		\
	fasl.l			\
	fixnums.l		\
	format.l		\
	funcall.l		\
	lambda.l		\
	lists.l			\
	macros.l		\
	maps.l			\
	parse.l			\
	read-macro.l		\
	read.l			\
	repl.l			\
	sequences.l		\
	streams.l		\
	strings.l		\
	symbol-macro.l		\
	symbols.l		\
	types.l         	\
	vectors.l

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
