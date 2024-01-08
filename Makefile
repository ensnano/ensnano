EXE=ensnano
MACOS_BIN=target/x86_64-apple-darwin/release/${EXE}
WINDOWS_BIN=target/x86_64-pc-windows-gnu/release/${EXE}
MACOS_M1_BIN=target/aarch64-apple-darwin/release/${EXE}

RELEASE_OPT= #--features=log_after_renderer_setup

SIGNATURE=Developer ID Application: Nicolas Schabanel (2KPHSEF9U9)
ICON_APP=app-icons/ENSnano

SHADER_SCENE=ensnano-scene/src/view
SHADERS_GRID=${SHADER_SCENE}/grid
SHADERS_CIRCLE2D=ensnano-utils/src/circles2d
SHADERS_CHARS2D=ensnano-utils/src/chars2d
SHADERS_MULTIPLEXER=src/multiplexer
SHADERS_FLATSCENE=ensnano-flatscene/src/view

SHADERS= ${SHADER_SCENE}/direction_cube.frag.spv ${SHADER_SCENE}/grid_disc.vert.spv ${SHADER_SCENE}/direction_cube.vert.spv ${SHADER_SCENE}/letter.frag.spv ${SHADER_SCENE}/dna_obj.frag.spv ${SHADER_SCENE}/letter.vert.spv ${SHADER_SCENE}/dna_obj.vert.spv ${SHADER_SCENE}/plane.frag.spv ${SHADER_SCENE}/dna_obj_fake.frag.spv ${SHADER_SCENE}/plane.vert.spv ${SHADER_SCENE}/dna_obj_outline.frag.spv ${SHADER_SCENE}/sheet_2d.frag.spv ${SHADER_SCENE}/dna_obj_outline.vert.spv ${SHADER_SCENE}/sheet_2d.vert.spv ${SHADER_SCENE}/gltf_obj.frag.spv ${SHADER_SCENE}/skybox.frag.spv ${SHADER_SCENE}/gltf_obj.vert.spv ${SHADER_SCENE}/skybox.vert.spv ${SHADER_SCENE}/grid_disc.frag.spv ${SHADERS_GRID}/grid.frag.spv ${SHADERS_GRID}/grid.vert.spv ${SHADERS_GRID}/texture.frag.spv ${SHADERS_GRID}/texture.vert.spv ${SHADERS_MULTIPLEXER}/draw.frag.spv ${SHADERS_MULTIPLEXER}/draw.vert.spv ${SHADERS_CIRCLE2D}/circle.frag.spv ${SHADERS_CIRCLE2D}/circle.vert.spv ${SHADERS_CIRCLE2D}/rotation_widget.frag.spv ${SHADERS_CHARS2D}/chars.frag.spv ${SHADERS_CHARS2D}/chars.vert.spv ${SHADERS_FLATSCENE}/border.frag.spv ${SHADERS_FLATSCENE}/rectangle.vert.spv ${SHADERS_FLATSCENE}/background.vert.spv ${SHADERS_FLATSCENE}/strand.frag.spv ${SHADERS_FLATSCENE}/insertion.vert.spv ${SHADERS_FLATSCENE}/grid.vert.spv ${SHADERS_FLATSCENE}/grid.frag.spv ${SHADERS_FLATSCENE}/strand.vert.spv ${SHADERS_FLATSCENE}/background.frag.spv ${SHADERS_FLATSCENE}/rectangle.frag.spv ${SHADERS_FLATSCENE}/border.vert.spv
# must rename plan_vert.spv plane_frag.spv viewborder.frag.spv

shaders: ${SHADERS}
	@echo Shaders compilation: Done, you should \"cargo clean\" to propagate the changes.

clean-shaders:
	rm ${SHADERS}

%.frag.spv: %.frag
	glslang -V $< -o $@

%.vert.spv: %.vert
	glslang -V $< -o $@

touch: 
	touch src

${MACOS_BIN}: 	export MACOSX_DEPLOYMENT_TARGET=10.13
${MACOS_M1_BIN}: export MACOSX_DEPLOYMENT_TARGET=11.0

${MACOS_BIN}: src
	@echo MACOSX_DEPLOYMENT_TARGET = $$MACOSX_DEPLOYMENT_TARGET
	cargo build --release --target x86_64-apple-darwin ${RELEASE_OPT}
	@echo "\n**** VERSION VERIFICATION ****\n"
	@otool -l ${MACOS_BIN} | grep -A 3 LC_VERSION_MIN_MACOSX || echo no match for LC_VERSION_MIN_MACOSX
	@otool -l ${MACOS_BIN} | grep -A 3 minos || echo no match for minos
	@echo "\n**** VERSION VERIFICATION ****\n"

all: mo m1 wingnu wingnudx12 winmsvc winmsvcdx12

rm_mo:
	rm ${MACOS_BIN}

wingnu:
	cargo build --release --target=x86_64-pc-windows-gnu ${RELEASE_OPT}
	cp ${WINDOWS_BIN}.exe ${WINDOWS_BIN}_windows_vulkan.exe

wingnudx12:
	cargo build --release --target=x86_64-pc-windows-gnu --features="dx12_only log_after_renderer_setup" ${RELEASE_OPT}
	cp ${WINDOWS_BIN}.exe ${WINDOWS_BIN}_windows_directx12.exe

winmsvc:
	cargo build --release --target=x86_64-pc-windows-msvc ${RELEASE_OPT}

winmsvcdx12:
	cargo build --release --target=x86_64-pc-windows-msvc --features="dx12_only log_after_renderer_setup" ${RELEASE_OPT}

mo: 
	make ${MACOS_BIN}
	cargo build --release --target x86_64-apple-darwin ${RELEASE_OPT}

mo_bt: 
	make ${MACOS_BIN}
	RUST_BACKTRACE=full cargo run --release --target x86_64-apple-darwin

mos: ${MACOS_BIN}
	@echo "App signing"
	xattr -cr ${MACOS_BIN}
	codesign -s "${SIGNATURE}" ${MACOS_BIN}
	@echo "\n**** Adding icon ****"
	sips -i ${ICON_APP}.icns
	DeRez -only icns ${ICON_APP}.icns > ${ICON_APP}_tmp.rsrc
	Rez -append ${ICON_APP}_tmp.rsrc -o ${MACOS_BIN}
	SetFile -a C ${MACOS_BIN}
	rm ${ICON_APP}_tmp.rsrc
	@echo "\n**** SIGNATURE VERIFICATION ****"
	codesign -dvvvv ${MACOS_BIN}



${MACOS_M1_BIN}: src
	@echo MACOSX_DEPLOYMENT_TARGET = $$MACOSX_DEPLOYMENT_TARGET
	cargo build --release --target aarch64-apple-darwin ${RELEASE_OPT}
	@echo "\n**** VERSION VERIFICATION ****\n"
	@otool -l ${MACOS_M1_BIN} | grep -A 3 LC_VERSION_MIN_MACOSX || echo no match for LC_VERSION_MIN_MACOSX
	@otool -l ${MACOS_M1_BIN} | grep -A 3 minos || echo no match for minos
	@echo "\n**** VERSION VERIFICATION ****\n"

m1: 
	make ${MACOS_M1_BIN}

m1s: m1
	@echo "\n**** APP SIGNING ****"
	xattr -cr ${MACOS_M1_BIN}
	codesign -s "${SIGNATURE}" ${MACOS_M1_BIN}
	@echo "\n**** ADDING ICON ****"
	#sips -i ${ICON_APP}.icns
	DeRez -only icns ${ICON_APP}.icns > ${ICON_APP}.tmp.rsrc
	Rez -append ${ICON_APP}.tmp.rsrc -o ${MACOS_M1_BIN}
	SetFile -a C ${MACOS_M1_BIN}
	rm ${ICON_APP}.tmp.rsrc
	@echo "\n**** SIGNATURE VERIFICATION ****"
	codesign -dvvvv ${MACOS_M1_BIN}
 



rm_m1:
	rm ${MACOS_M1_BIN}


${WINDOWS_BIN}: src
	cargo build --release --target x86_64-pc-windows-gnu

win: 
	make ${WINDOWS_BIN}

org:
	@echo Updating organizer...
	cargo update -p ensnano_organizer
	@echo ... Done
