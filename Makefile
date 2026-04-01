VULKAN_SHADER_COMPILER=glslang -V
EXE=ensnano
MACOS_BIN=target/x86_64-apple-darwin/release/$(EXE)
WINDOWS_BIN=target/x86_64-pc-windows-gnu/release/$(EXE)
WINDOWS_BIN_AARCH=target/aarch64-pc-windows-msvc/release/$(EXE)
MACOS_M1_BIN=target/aarch64-apple-darwin/release/$(EXE)

RELEASE_OPT= #--features=log_after_renderer_setup

SIGNATURE=Developer ID Application: Nicolas Schabanel (2KPHSEF9U9)
ICON_APP=app-icons/ENSnano

FRAG_SRCS := $(shell find . -name '*.frag')
VERT_SRCS := $(shell find . -name '*.vert')
SHADERS := $(patsubst %.frag,%.frag.spv,$(FRAG_SRCS)) $(patsubst %.vert,%.vert.spv,$(VERT_SRCS))

validate:
	@rustup update stable
	@$(MAKE) -s shaders
	@$(MAKE) -s format
	@$(MAKE) -s spell # If the command fails, install npm
	@cargo machete # If the command fails: cargo install cargo-machete
	@RUSTFLAGS="--deny warnings" $(MAKE) -s check
	@RUSTFLAGS="--deny warnings" $(MAKE) -s lint
	@$(MAKE) -s test

check:
	@cargo check --workspace --all-targets

lint:
	@cargo clippy --workspace --all-targets

# does not include doctests (--doc), but we have none so far
test:
	@cargo test --workspace --all-targets --release

format:
	@cargo fmt --all

spell:
	@npx cspell .

shaders: $(SHADERS)
	@echo Shaders compilation: Done.

clean-shaders:
	rm -f $(SHADERS)

re-shaders:
	$(MAKE) clean-shaders
	$(MAKE) shaders

clean: re-shaders
	rm -f ensnano-scene/*.stl
	rm -f *.dot

%.frag.spv: %.frag
	$(VULKAN_SHADER_COMPILER) $< -o $@

%.vert.spv: %.vert
	$(VULKAN_SHADER_COMPILER) $< -o $@

touch: 
	touch src

$(MACOS_BIN): 	export MACOSX_DEPLOYMENT_TARGET=10.13
$(MACOS_M1_BIN): export MACOSX_DEPLOYMENT_TARGET=11.0

$(MACOS_BIN): src
	@echo MACOSX_DEPLOYMENT_TARGET = $$MACOSX_DEPLOYMENT_TARGET
	cargo build --release --target x86_64-apple-darwin $(RELEASE_OPT)
	@echo "\n**** VERSION VERIFICATION ****\n"
	@otool -l $(MACOS_BIN) | grep -A 3 LC_VERSION_MIN_MACOSX || echo no match for LC_VERSION_MIN_MACOSX
	@otool -l $(MACOS_BIN) | grep -A 3 minos || echo no match for minos
	@echo "\n**** VERSION VERIFICATION ****\n"

all: mo m1 wingnu winarm wingnudx12 winmsvc winmsvcdx12

rm_mo:
	rm $(MACOS_BIN)

wingnu:
	cargo build --release --target=x86_64-pc-windows-gnu $(RELEASE_OPT)
	cp $(WINDOWS_BIN).exe $(WINDOWS_BIN)_windows_vulkan.exe

winarm:
	cargo xwin build --release --target aarch64-pc-windows-msvc $(RELEASE_OPT)
	cp $(WINDOWS_BIN_AARCH).exe $(WINDOWS_BIN_AARCH)_windows_vulkan_aarch64.exe

wingnudx12:
	cargo build --release --target=x86_64-pc-windows-gnu --features="dx12_only log_after_renderer_setup" $(RELEASE_OPT)
	cp $(WINDOWS_BIN).exe $(WINDOWS_BIN)_windows_directx12.exe

winmsvc:
	cargo build --release --target=x86_64-pc-windows-msvc $(RELEASE_OPT)

winmsvcdx12:
	cargo build --release --target=x86_64-pc-windows-msvc --features="dx12_only log_after_renderer_setup" $(RELEASE_OPT)

mo: 
	make $(MACOS_BIN)
	cargo build --release --target x86_64-apple-darwin $(RELEASE_OPT)

mo_bt: 
	make $(MACOS_BIN)
	RUST_BACKTRACE=full cargo run --release --target x86_64-apple-darwin

mos: $(MACOS_BIN)
	@echo "App signing"
	xattr -cr $(MACOS_BIN)
	codesign -s "$(SIGNATURE)" $(MACOS_BIN)
	@echo "\n**** Adding icon ****"
	sips -i $(ICON_APP).icns
	DeRez -only icns $(ICON_APP).icns > $(ICON_APP)_tmp.rsrc
	Rez -append $(ICON_APP)_tmp.rsrc -o $(MACOS_BIN)
	SetFile -a C $(MACOS_BIN)
	rm $(ICON_APP)_tmp.rsrc
	@echo "\n**** SIGNATURE VERIFICATION ****"
	codesign -dvvvv $(MACOS_BIN)

$(MACOS_M1_BIN): src
	@echo MACOSX_DEPLOYMENT_TARGET = $$MACOSX_DEPLOYMENT_TARGET
	cargo build --release --target aarch64-apple-darwin $(RELEASE_OPT)
	@echo "\n**** VERSION VERIFICATION ****\n"
	@otool -l $(MACOS_M1_BIN) | grep -A 3 LC_VERSION_MIN_MACOSX || echo no match for LC_VERSION_MIN_MACOSX
	@otool -l $(MACOS_M1_BIN) | grep -A 3 minos || echo no match for minos
	@echo "\n**** VERSION VERIFICATION ****\n"

m1: 
	make $(MACOS_M1_BIN)

m1s: m1
	@echo "\n**** APP SIGNING ****"
	xattr -cr $(MACOS_M1_BIN)
	codesign -s "$(SIGNATURE)" $(MACOS_M1_BIN)
	@echo "\n**** ADDING ICON ****"
	#sips -i $(ICON_APP).icns
	DeRez -only icns $(ICON_APP).icns > $(ICON_APP).tmp.rsrc
	Rez -append $(ICON_APP).tmp.rsrc -o $(MACOS_M1_BIN)
	SetFile -a C $(MACOS_M1_BIN)
	rm $(ICON_APP).tmp.rsrc
	@echo "\n**** SIGNATURE VERIFICATION ****"
	codesign -dvvvv $(MACOS_M1_BIN)

rm_m1:
	rm $(MACOS_M1_BIN)

$(WINDOWS_BIN): src
	cargo build --release --target x86_64-pc-windows-gnu

win: 
	make $(WINDOWS_BIN)

win-aarch:
	make $(WINDOWS_BIN_AARCH)

upcoming:
	mv -i Cargo.toml Cargo.toml.saved
	cat Cargo.toml.prefix Cargo.toml.middle.upcoming Cargo.toml.suffix > Cargo.toml
	mv -i ensnano-design/Cargo.toml ensnano-design/Cargo.toml.saved
	cat ensnano-design/Cargo.toml.prefix ensnano-design/Cargo.toml.middle.upcoming ensnano-design/Cargo.toml.suffix > ensnano-design/Cargo.toml

noupcoming:
	mv -i Cargo.toml Cargo.toml.saved
	cat Cargo.toml.prefix Cargo.toml.middle.noupcoming Cargo.toml.suffix > Cargo.toml
	mv -i ensnano-design/Cargo.toml ensnano-design/Cargo.toml.saved
	cat ensnano-design/Cargo.toml.prefix ensnano-design/Cargo.toml.middle.noupcoming ensnano-design/Cargo.toml.suffix > ensnano-design/Cargo.toml

