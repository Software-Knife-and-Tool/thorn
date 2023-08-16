#
#  tools makefile
#
PHONY: core-xref help

help:
	@echo "core-xref:    core cross reference"

core-xref:
	@thorn --pipe --load=./core-xref.l --quiet-eval='(core::xref \"core-xref.out\")'
	@python3 core-xref.py core-xref.out

clean:
	@rm -f core-xref.out
