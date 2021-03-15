PROJECT=grid-world

.PHONY: all clean debug help release

##all     : Builds the project from scratch in release mode.
all: clean release

build:
	mkdir $@

build/debug: | build
	mkdir $@

build/release: | build
	mkdir $@

# Builds the game in debug mode.
target/debug/$(PROJECT): $(shell find $(PROJECT)/src)
	cargo build

# Builds the game in release mode.
target/release/$(PROJECT): $(shell find $(PROJECT)/src)
	cargo build --release

##debug   : Packages the game and its content in debug mode.
debug: target/debug/$(PROJECT) | build/debug
	rm -rf build/debug/*
	cp $< ./build/debug

##release : Packages the game and its content in release mode.
release: target/release/$(PROJECT) | build/release
	rm -rf build/release/*
	cp $< ./build/release

##dev     : Starts the packaged game in debug mode.
dev: debug
	cd ./build/debug; ./$(PROJECT)

##clean   : Recursively removes any directories that were created during the make process.
clean:
	if [ -d "./build" ]; then rm -r build; fi

##help    : Displays information regarding some of this Makefile's rule.
help: Makefile
	@sed -n 's/^##//p' $<
