#
#  tools makefile
#
.PHONY: crossref symbols help

help:
	@echo "crossref: thorn symbol cross reference"
	@echo "symbols: thorn symbol info"

crossref:
	@thorn --pipe --load=./crossref.l --quiet-eval='(prelude:crossref \"crossref.out\")'
	@python3 crossref.py crossref.out

symbols:
	@thorn --pipe --load=./symbols.l --quiet-eval='(prelude:symbols \"symbols.out\")'
	@python3 symbols.py ../dist/prelude.l symbols.out

clean:
	@rm -f crossref.out symbols.out
