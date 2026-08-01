# Makefile for a Cargo project

SHELL := /bin/bash
.ONESHELL:           # run all lines of a target in a single shell
.DEFAULT_GOAL := help

# Colors for help
BLUE := \033[34m
RESET := \033[0m

ifeq ($(OS),Windows_NT)
    DETECTED_OS := Windows
else
    # Use uname -s to get the system name (Linux, Darwin, MINGW*, CYGWIN*, etc.)
    DETECTED_OS := $(shell uname -s)
endif

.PHONY: help
help:
	@printf "Available targets:\n"
	@printf "  $(BLUE)make build$(RESET)                          						Build the project (debug mode)\n"
	@printf "  $(BLUE)make run$(RESET)                            						Run the project (debug mode)\n"
	@printf "  $(BLUE)make build.release$(RESET)                   						Build the project in release mode (optimized)\n"
	@printf "  $(BLUE)make run.release$(RESET)                     						Run the project in release mode\n"
	@printf "  $(BLUE)make example-build FILE=<name>$(RESET)       						Build an example (debug mode)\n"
	@printf "  $(BLUE)make example-build-release FILE=<name>$(RESET)  					Build an example in release mode\n"
	@printf "  $(BLUE)make example-run FILE=<name>$(RESET)         						Run an example (debug mode)\n"
	@printf "  $(BLUE)make example-run-release FILE=<name>$(RESET)  					Run an example in release mode\n"
	@printf "  $(BLUE)make example-java-build $(RESET)                   					Build the Java example (examples/java/build)\n"
	@printf "  $(BLUE)make clear-target$(RESET)                    						Remove entire target directory\n"
	@printf "  $(BLUE)make clear-target-debug$(RESET)              						Remove debug build artifacts\n"
	@printf "  $(BLUE)make clear-target-release$(RESET)            						Remove release build artifacts\n"
	@printf "  $(BLUE)make clear-target-debug-example$(RESET)      						Remove debug example binaries\n"
	@printf "  $(BLUE)make clear-target-release-example$(RESET)    						Remove release example binaries\n"
	@printf "  $(BLUE)make clear-doc$(RESET)                       						Remove generated documentation\n"
	@printf "  $(BLUE)make doc$(RESET)                             						Generate documentation (without dependencies)\n"
	@printf "  $(BLUE)make doc-open$(RESET)                        						Generate and open documentation in browser\n"
	@printf "  $(BLUE)make add CRATE=<crate> [VERSION=<ver>] [FEATURES=<feat>]$(RESET)			Add a dependency\n"
	@printf "  $(BLUE)make remove CRATE=<crate>$(RESET)            						Remove a dependency\n"
	@printf "  $(BLUE)make update$(RESET)                          						Update dependencies as per Cargo.lock\n"
	@printf "  $(BLUE)make check$(RESET)                           						Check the project for errors (without building)\n"
	@printf "  $(BLUE)make version$(RESET)                         						Show versions of Rust toolchain components\n"

.PHONY: build
build:
	cargo build

.PHONY: run
run:
	cargo run

.PHONY: build.release
build.release:
	cargo build --release

.PHONY: run.release
run.release:
	cargo run --release

# Example targets – require FILE=<name>
.PHONY: example-build
example-build:
	@[ -n "$(FILE)" ] || (echo "ERROR: Please specify FILE=<filename>" && exit 1)
	cargo build --example $(FILE)

.PHONY: example-build-release
example-build-release:
	@[ -n "$(FILE)" ] || (echo "ERROR: Please specify FILE=<filename>" && exit 1)
	cargo build --example $(FILE) --release

.PHONY: example-run
example-run:
	@[ -n "$(FILE)" ] || (echo "ERROR: Please specify FILE=<filename>" && exit 1)
	./target/debug/examples/$(FILE)

.PHONY: example-run-release
example-run-release:
	@[ -n "$(FILE)" ] || (echo "ERROR: Please specify FILE=<filename>" && exit 1)
	./target/release/examples/$(FILE)

.PHONY: example-java-build
example-java-build:
	cd examples/java && ./build.sh

# Clean targets
.PHONY: clear-target
clear-target:
	rm -rf target

.PHONY: clear-target-debug
clear-target-debug:
	rm -rf target/debug

.PHONY: clear-target-release
clear-target-release:
	rm -rf target/release

.PHONY: clear-target-debug-example
clear-target-debug-example:
	rm -rf target/debug/examples

.PHONY: clear-target-release-example
clear-target-release-example:
	rm -rf target/release/examples

.PHONY: clear-doc
clear-doc:
	rm -rf target/doc

# Documentation
.PHONY: doc
doc:
	cargo doc --no-deps

.PHONY: doc-open
doc-open:
	cargo doc --no-deps --open

# Crate management
.PHONY: add
add:
	@[ -n "$(CRATE)" ] || (echo "ERROR: Please specify CRATE=<crate>" && exit 1)
	cmd="cargo add $(CRATE)"
	if [ -n "$(VERSION)" ]; then cmd+=" @$(VERSION)"; fi
	if [ -n "$(FEATURES)" ]; then cmd+=" --features $(FEATURES)"; fi
	eval $$cmd

.PHONY: remove
remove:
	@[ -n "$(CRATE)" ] || (echo "ERROR: Please specify CRATE=<crate>" && exit 1)
	cargo remove $(CRATE)

.PHONY: update
update:
	cargo update

.PHONY: check
check:
	cargo check

.PHONY: version
version:
	@rustup --version 2>/dev/null || echo "rustup not installed"
	@rustc --version
	@cargo --version