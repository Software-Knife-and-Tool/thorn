#
#  tools makefile
#
PHONY: crossref revcrossref help

help:
	@echo "crossref: generate core cross reference"
	@echo "revcrossref: generate core reverse cross reference"

crossref:
	@thorn --pipe --load=./crossref.l --eval='(core::crossref \"crossref.out\")'
	@python3 crossref.py crossref.out

revcrossref:
	@thorn --pipe --load=./crossref.l --eval='(core::crossref \"crossref.out\")'
	@python3 revcrossref.py crossref.out

clean:
	@rm -f crossref.out
