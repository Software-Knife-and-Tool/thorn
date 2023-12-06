#
# thorn makefile
#
.PHONY: thorn core
SRC = ../src

# core.l needs to be first
CORE = \
	core.l		\
	backquote.l	\
	break.l		\
	boole.l		\
	compile.l	\
	ctype.l		\
	describe.l	\
	environment.l	\
	exception.l	\
	fasl.l		\
	fixnum.l	\
	format.l	\
	funcall.l	\
	function.l	\
	inspect.l	\
	lambda.l	\
	list.l		\
	log.l		\
	macro.l		\
	map.l		\
	namespace.l	\
	parse.l		\
	read-macro.l	\
	read.l		\
	repl.l		\
	sequence.l	\
	sort.l		\
	stream.l	\
	string.l	\
	symbol-macro.l	\
	symbol.l	\
	time.l		\
	type.l		\
	vector.l

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
