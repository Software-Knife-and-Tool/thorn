#
# thorn makefile
#
.PHONY: world run clean doc dist format tests runtime release
RUNTIME=target/debug/runtime

help:
	@echo "thorn top-level makefile -----------------"
	@echo
	@echo "--- build options"
	@echo "    world - build release and package dist"
	@echo "    debug - build runtime for debug"
	@echo "    release - build runtime for release"
	@echo "--- distribution options"
	@echo "    doc - generate documentation"
	@echo "    dist - build distribution image"
	@echo "    install - install distribution (needs sudo)"
	@echo "    uninstall - uninstall distribution (needs sudo)"
	@echo "--- development options"
	@echo "    clean - remove build artifacts"
	@echo "    commit - run clippy, rustfmt, make test and perf reports"
	@echo "    report - view test and perf reports"
	@echo "    tags - make etags"
	@echo "--- test options"
	@echo "    tests/rust - rust tests"
	@echo "    tests/summary - test summary"
	@echo "    tests/report - full test report"
	@echo "--- perf options"
	@echo "    perf/base - baseline report"
	@echo "    perf/current - current report"
	@echo "    perf/diff - compare abseline and current"
	@echo "    perf/commit - condensed report"

world: release dist

tags:
	@etags `find src/mu -name '*.rs' -print`

release:
	@cargo build --release --features async
	@cp target/release/runtime dist

no-async:
	@cargo build --release --features no-async
	@cp target/release/runtime dist

debug:
	@cargo build
	@cp target/debug/runtime dist

dist:
	@make -C ./dist --no-print-directory

doc:
	@cargo doc
	@mkdir -p ./doc/rustdoc
	@cp -r ./target/doc/* ./doc/rustdoc
	@make -C ./doc --no-print-directory

install:
	@make -C ./dist -f install.mk install --no-print-directory

uninstall:
	@make -C ./dist -f install.mk uninstall --no-print-directory

tests/commit:
	@make -C tests commit --no-print-directory

tests/summary:
	@make -C tests summary --no-print-directory

perf/base:
	@make -C perf base --no-print-directory

perf/current:
	@make -C perf current --no-print-directory

perf/diff:
	@make -C perf diff --no-print-directory

perf/commit:
	@make -C perf commit --no-print-directory

commit:
	@cargo fmt
	@echo ";;; rust tests"
	@cargo -q test --features async | sed -e '/^$$/d'
	@cargo -q test --features no-async | sed -e '/^$$/d'
	@echo ";;; clippy tests"
	@cargo clippy --features async
	@cargo clippy --features no-async
	@make -C tests commit --no-print-directory
	@make -C perf commit --no-print-directory

clobber:
	@rm -f TAGS
	@make -C docker clean --no-print-directory
	@make -C dist clean --no-print-directory
	@make -C tests clean --no-print-directory
	@make -C perf clean --no-print-directory
