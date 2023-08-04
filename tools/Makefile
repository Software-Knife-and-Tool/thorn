#
#  tools makefile
#
PHONY: crossref help

help:
	@echo "crossref: generate core cross reference"

crossref:
	@thorn --pipe --load=./crossref.l --eval='(core::crossref \"crossref.out\")'
	@python3 crossref.py crossref.out

clean:
	@rm -f crossref.out
