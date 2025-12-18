BINARY := tbl
TARGET_LINUX := x86_64-unknown-linux-musl
TARGET_MACOS_ARM := aarch64-apple-darwin
TARGET_MACOS_X86 := x86_64-apple-darwin

.PHONY: all build release static static-linux clean help

all: release

# Development build
build:
	cargo build

# Release build (native)
release:
	cargo build --release

# Static Linux build (requires musl toolchain)
static: static-linux

static-linux:
	rustup target add $(TARGET_LINUX) 2>/dev/null || true
	cargo build --release --target $(TARGET_LINUX)
	mkdir -p dist
	cp target/$(TARGET_LINUX)/release/$(BINARY) dist/$(BINARY)-linux-x86_64

# macOS builds (for cross-compilation or native)
static-macos-arm:
	rustup target add $(TARGET_MACOS_ARM) 2>/dev/null || true
	cargo build --release --target $(TARGET_MACOS_ARM)
	mkdir -p dist
	cp target/$(TARGET_MACOS_ARM)/release/$(BINARY) dist/$(BINARY)-macos-arm64

static-macos-x86:
	rustup target add $(TARGET_MACOS_X86) 2>/dev/null || true
	cargo build --release --target $(TARGET_MACOS_X86)
	mkdir -p dist
	cp target/$(TARGET_MACOS_X86)/release/$(BINARY) dist/$(BINARY)-macos-x86_64

# Run development server
run:
	cargo run -- --no-browser

# Run with auto-reload (requires cargo-watch)
watch:
	cargo watch -x 'run -- --no-browser'

# Clean build artifacts
clean:
	cargo clean
	rm -rf dist

# Show help
help:
	@echo "tbl - Tiny self-bootstrapping web launcher"
	@echo ""
	@echo "Targets:"
	@echo "  build          - Development build"
	@echo "  release        - Release build (native)"
	@echo "  static         - Static Linux build (musl)"
	@echo "  static-linux   - Static Linux x86_64 build"
	@echo "  static-macos-arm - macOS ARM64 build"
	@echo "  static-macos-x86 - macOS x86_64 build"
	@echo "  run            - Run development server"
	@echo "  watch          - Run with auto-reload"
	@echo "  clean          - Clean build artifacts"
	@echo "  help           - Show this help"

