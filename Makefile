# Build
.PHONY: build
build: build_release cp_build

.PHONY: build_release
build_release:
	cd ./build && ./target/release/build_restore_file_info

.PHONY: cp_build
cp_build:
	cd ./build && mkdir -p ../dist && cp -rf ./bin/* ../dist

.PHONY: build_tools
build_tools:
	cd ./build && cargo build --release