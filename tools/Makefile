#
#  tools makefile
#
.PHONY: prelude-xref prelude-symbols help

help:
	@echo "prelude-xref: prelude cross reference"
	@echo "prelude-symbols: prelude symbol info"

prelude-xref:
	@thorn --pipe --load=./prelude-xref.l --quiet-eval='(prelude:xref \"prelude-xref.out\")'
	@python3 prelude-xref.py prelude-xref.out

prelude-symbols:
	@thorn --pipe --load=./prelude-symbols.l --quiet-eval='(prelude:symbols \"prelude-symbols.out\")'
	@python3 prelude-symbols.py ../dist/prelude.l prelude-symbols.out

clean:
	@rm -f prelude-xref.out prelude-symbols.out
