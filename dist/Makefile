#
# dist makefile
#
.PHONY: dist clean

BASE = thorn
DOCS = ../doc
DIST = ../dist
LOCAL = ./local
SERVER = ./server
VERSION = 0.0.8

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

	@install -m 755 $(LOCAL) $(BASE)/bin
	@install -m 755 $(SERVER) $(BASE)/bin
	@install -m 755 ./thorn.sh $(BASE)/bin/thorn

	@install -d $(BASE)/doc/html
	@install -m 644 $(DOCS)/refcards/*.pdf $(BASE)/doc
	@install -m 644 $(DOCS)/reference/html/*.html $(BASE)/doc/html

	@tar cfz $(BASE)-$(VERSION).tgz $(BASE)
	@rm -rf $(BASE)

clean:
	@rm -rf $(BASE)-$(VERSION).tgz $(BASE) $(LOCAL) $(SERVER)
