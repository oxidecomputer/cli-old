# Set the shell
SHELL := /bin/bash

# Set an output prefix, which is the local directory if not specified
PREFIX?=$(shell pwd)

NAME := oxide

# Set the build dir, where built cross-compiled binaries will be output
BUILDDIR := ${PREFIX}/cross

GENERATED_DOCS_DIR := ${PREFIX}/generated_docs

UNAME := $(shell uname)

# These are chosen from: https://doc.rust-lang.org/nightly/rustc/platform-support.html
ifeq ($(UNAME), Darwin)
	CROSS_TARGETS := x86_64-apple-darwin \
				 aarch64-apple-darwin
else
	CROSS_TARGETS := x86_64-pc-windows-gnu \
				 x86_64-unknown-linux-musl \
				 aarch64-unknown-linux-musl
	# Turn this back on when it works.
	# x86_64-unknown-illumos
	# i686-pc-windows-gnu
	# x86_64-unknown-freebsd
endif

# For this to work, you need to install toml-cli: https://github.com/gnprice/toml-cli
# `cargo install toml-cli`
VERSION := $(shell toml get $(CURDIR)/Cargo.toml package.version | jq -r .)

GITCOMMIT := $(shell git rev-parse --short HEAD)
GITUNTRACKEDCHANGES := $(shell git status --porcelain --untracked-files=no)
ifneq ($(GITUNTRACKEDCHANGES),)
	GITCOMMIT := $(GITCOMMIT)-dirty
endif
ifeq ($(GITCOMMIT),)
    GITCOMMIT := ${GITHUB_SHA}
endif

define buildrelease
rustup target add $(1)
cargo build --release --target $(1) || cross build --release --target $(1)
mv $(CURDIR)/target/$(1)/release/$(NAME) $(BUILDDIR)/$(NAME)-$(1) || mv $(CURDIR)/target/$(1)/release/$(NAME).exe $(BUILDDIR)/$(NAME)-$(1)
md5sum $(BUILDDIR)/$(NAME)-$(1) > $(BUILDDIR)/$(NAME)-$(1).md5;
sha256sum $(BUILDDIR)/$(NAME)-$(1) > $(BUILDDIR)/$(NAME)-$(1).sha256;
echo -e "### $(1)\n\n" >> $(BUILDDIR)/README.md;
echo -e "\`\`\`console" >> $(BUILDDIR)/README.md;
echo -e "# Export the sha256sum for verification." >> $(BUILDDIR)/README.md;
echo -e "$$ export OXIDE_CLI_SHA256=\"`cat $(BUILDDIR)/$(NAME)-$(1).sha256 | awk '{print $$1}'`\"\n\n" >> $(BUILDDIR)/README.md;
echo -e "# Download and check the sha256sum." >> $(BUILDDIR)/README.md;
echo -e "$$ curl -fSL \"https://dl.oxide.computer/releases/cli/v$(VERSION)/$(NAME)-$(1)\" -o \"/usr/local/bin/oxide\" \\" >> $(BUILDDIR)/README.md;
echo -e "\t&& echo \"\$${OXIDE_CLI_SHA256}  /usr/local/bin/oxide\" | sha256sum -c - \\" >> $(BUILDDIR)/README.md;
echo -e "\t&& chmod a+x \"/usr/local/bin/oxide\"\n\n" >> $(BUILDDIR)/README.md;
echo -e "$$ echo \"oxide cli installed!\"\n" >> $(BUILDDIR)/README.md;
echo -e "# Run it!" >> $(BUILDDIR)/README.md;
echo -e "$$ oxide -h" >> $(BUILDDIR)/README.md;
echo -e "\`\`\`\n\n" >> $(BUILDDIR)/README.md;
endef

# If running on a Mac you will need:
# 	brew install filosottile/musl-cross/musl-cross
.PHONY: release
release: src/*.rs Cargo.toml ## Builds the cross-compiled binaries, naming them in such a way for release (eg. binary-OS-ARCH).
	@echo "+ $@"
	mkdir -p $(BUILDDIR)
	$(foreach TARGET,$(CROSS_TARGETS), $(call buildrelease,$(TARGET)))

.PHONY: tag
tag: ## Create a new git tag to prepare to build a release.
	git tag -sa v$(VERSION) -m "v$(VERSION)"
	@echo "Run git push origin v$(VERSION) to push your new tag to GitHub and trigger a release."

.PHONY: AUTHORS
AUTHORS:
	@$(file >$@,# This file lists all individuals having contributed content to the repository.)
	@$(file >>$@,# For how it is generated, see `make AUTHORS`.)
	@echo "$(shell git log --format='\n%aN <%aE>' | LC_ALL=C.UTF-8 sort -uf)" >> $@

.PHONY: clean
clean: ## Cleanup any build binaries or packages.
	@echo "+ $@"
	$(RM) -r $(BUILDDIR)
	$(RM) -r $(GENERATED_DOCS_DIR)

build: Cargo.toml $(wildcard src/*.rs) ## Build the Rust crate.
	cargo build


.PHONY: start-cockroachdb
start-cockroachdb: ## Start CockroachDB.
	@echo "+ $@"
	@docker rm -f cockroachdb || true
	docker run -d \
		--restart=always \
		--name=cockroachdb \
		--hostname=cockroachdb \
		-p 0.0.0.0:26257:26257 \
		-p 0.0.0.0:1234:8080  \
		cockroachdb/cockroach:v21.2.1 start-single-node \
			--insecure
	@echo "Waiting for CockroachDB to start..."
	@sleep 5

OMICRON_DOCKER_VERSION:=ci

.PHONY: start-omicron
start-omicron: start-cockroachdb ## Start Omicron.
	@echo "+ $@"
	@docker rm -f nexus || true
	@docker rm -f sled-agent || true
	@echo "Populating the database for omicron...."
	docker run --rm -i \
		--name=bootstrap_db \
		--hostname=nexus \
		--net host \
		--entrypoint=omicron-dev \
		ghcr.io/oxidecomputer/omicron:$(OMICRON_DOCKER_VERSION) \
			db-populate --database-url "postgresql://root@0.0.0.0:26257/omicron?sslmode=disable"
	@echo "Starting nexus..."
	docker run -d \
		--restart=always \
		--name=nexus \
		--hostname=nexus \
		--net host \
		-v "$(CURDIR)/tests/omicron.toml:/etc/omicron/config.toml:ro"  \
		--entrypoint=nexus \
		ghcr.io/oxidecomputer/omicron:$(OMICRON_DOCKER_VERSION) \
			/etc/omicron/config.toml
	@echo "Starting sled-agent..."
	docker run -d \
		--restart=always \
		--name=sled-agent \
		--hostname=sled-agent \
		--net host \
		--entrypoint=sled-agent-sim \
		ghcr.io/oxidecomputer/omicron:$(OMICRON_DOCKER_VERSION) \
			B100B75C-D2EF-415F-A07E-D3915470913D 0.0.0.0:12345 0.0.0.0:12221

.PHONY: gen-docs
gen-docs: gen-md gen-man ## Generate all the docs.

.PHONY: gen-md
gen-md: build  ## Generate the markdown documentation.
	$(CURDIR)/target/debug/oxide generate markdown --dir $(GENERATED_DOCS_DIR)/md

.PHONY: gen-man
gen-man: build ## Generate the man pages.
	$(CURDIR)/target/debug/oxide generate man-pages --dir $(GENERATED_DOCS_DIR)/man

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | sed 's/^[^:]*://g' | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

check_defined = \
    $(strip $(foreach 1,$1, \
	$(call __check_defined,$1,$(strip $(value 2)))))

__check_defined = \
    $(if $(value $1),, \
    $(error Undefined $1$(if $2, ($2))$(if $(value @), \
    required by target `$@')))

