#
#  tools makefile
#
PHONY: xref revxref help

help:
	@echo "xref:    core cross reference"
	@echo "revxref: core reverse cross reference"

xref:
	@thorn --pipe --load=./xref.l --quiet-eval='(core::xref \"xref.out\")'
	@python3 xref.py xref.out

revxref:
	@thorn --pipe --load=./xref.l --quiet-eval='(core::xref \"xref.out\")'
	@python3 revxref.py xref.out

clean:
	@rm -f xref.out
