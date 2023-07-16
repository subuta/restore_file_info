# Build
.PHONY: build
build: build_release cp_build

.PHONY: build_release
build_release:
	cargo build --release

.PHONY: cp_build
cp_build:
	mkdir -p ./dist
	cp ./target/release/restore_file_info ./dist/restore_file_info
