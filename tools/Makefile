#
#  tools makefile
#
.PHONY: core-xref core-symbols help

help:
	@echo "core-xref: core cross reference"
	@echo "core-symbols: core symbol info"

core-xref:
	@thorn --pipe --load=./core-xref.l --quiet-eval='(core:xref \"core-xref.out\")'
	@python3 core-xref.py core-xref.out

core-symbols:
	@thorn --pipe --load=./core-symbols.l --quiet-eval='(core:symbols \"core-symbols.out\")'
	@python3 core-symbols.py ../dist/core.l core-symbols.out

clean:
	@rm -f core-xref.out core-symbols.out
