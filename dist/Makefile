#
# dist makefile
#
.PHONY: dist clean

VERSION = 0.0.28

BASE = thorn
DOCS = ../doc
DIST = ../dist
MU_SHELL = ./mu-shell
MU_SERVER = ./mu-server

dist:
	@install -d $(BASE)
	@install -d $(BASE)/thorn
	@install -d $(BASE)/thorn/core
	@install -d $(BASE)/thorn/preface
	@install -d $(BASE)/thorn/mu

	@make -f thorn.mk thorn core

	@install -d $(BASE)/bin
	@install -d $(BASE)/$(BASE)
	@install -d $(BASE)/doc

	@install -m 644 $(DIST)/thorn.l $(BASE)/$(BASE)

	@install -m 755 $(MU_SHELL) $(BASE)/bin
	@install -m 755 $(MU_SERVER) $(BASE)/bin
	@install -m 755 ./thorn.sh $(BASE)/bin/thorn

	@install -d $(BASE)/doc/html
	@install -m 644 $(DOCS)/refcards/*.pdf $(BASE)/doc
	@install -m 644 $(DOCS)/reference/html/*.html $(BASE)/doc/html

	@tar cfz $(BASE)-$(VERSION).tgz $(BASE)
	@rm -rf $(BASE)

clean:
	@rm -rf $(BASE)-$(VERSION).tgz $(BASE) $(MU_SHELL) $(MU_SERVER)
