prog :=hawk
debug ?=

ifdef debug
	release :=
	target :=debug
else
	release :=--release
	target :=release
endif

build:
	cargo build $(release)

install:
	mv ./target/$(target)/$(prog) /usr/local/bin/$(prog)

tar:
	cargo build --release;
	tar -czf $(prog).tar.gz --directory=./target/$(target) $(prog)
	shasum -a 256 $(prog).tar.gz

tag:
	git tag -a v$(version) -m "version $(version)"
	git push --tags

help:
	@echo "usage: make $(prog) [debug=1]"
