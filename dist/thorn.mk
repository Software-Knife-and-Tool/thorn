#
# thorn makefile
#
.PHONY: thorn prelude
SRC = ../src

# prelude.l needs to be first
PRELUDE = \
	prelude.l	\
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

thorn:
	@cp -r $(SRC)/prelude thorn/thorn
	@cp -r $(SRC)/mu thorn/thorn

prelude:
	@rm -f prelude.l
	@for prelude in $(PRELUDE); do		\
	    cat $(SRC)/prelude/$$prelude >> prelude.l;	\
	done
