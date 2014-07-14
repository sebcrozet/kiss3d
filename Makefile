tmp=_git_distcheck
kiss3d_lib_path=lib
kiss3d_bin_path=bin
kiss3d_doc_path=doc
glfw_path=lib/glfw-rs
glfw_lib_path=lib/glfw-rs/lib
gl_lib_path=lib/gl-rs/lib
nalgebra_lib_path=lib/nalgebra/lib
stb_image_lib_path=lib/rust-stb-image/
freetype_path=lib/rust-freetype/
ffmpeg_path=lib/rust-ffmpeg/lib
ncollide_path=lib/ncollide/lib
libs=-L$(glfw_lib_path) -L$(gl_lib_path) -L$(nalgebra_lib_path) -L$(stb_image_lib_path) -L$(freetype_path) -L$(ffmpeg_path) -L$(ncollide_path)
build_cmd= rustc -Llib  $(libs) --opt-level 3 --out-dir $(kiss3d_bin_path)

all: kiss3d

kiss3d:
	mkdir -p $(kiss3d_lib_path)
	rustc src/lib.rs --crate-type dylib --crate-type rlib --opt-level 3 --out-dir $(kiss3d_lib_path) $(libs)

kiss3d_tools: deps_recording
	mkdir -p $(kiss3d_lib_path)
	rustc src/tools/kiss3d_recording.rs --crate-type dylib --crate-type rlib --opt-level 3 -L lib --out-dir $(kiss3d_lib_path) $(libs)

test: examples

examples:
	mkdir -p $(kiss3d_bin_path)
	$(build_cmd) ./examples/procedural.rs 
	$(build_cmd) ./examples/relativity.rs 
	$(build_cmd) ./examples/camera.rs 
	$(build_cmd) ./examples/event.rs 
	$(build_cmd) ./examples/cube.rs 
	$(build_cmd) ./examples/add_remove.rs 
	$(build_cmd) ./examples/custom_material.rs 
	$(build_cmd) ./examples/custom_mesh.rs 
	$(build_cmd) ./examples/custom_mesh_shared.rs 
	$(build_cmd) ./examples/group.rs 
	$(build_cmd) ./examples/lines.rs 
	$(build_cmd) ./examples/obj.rs 
	$(build_cmd) ./examples/points.rs
	$(build_cmd) ./examples/post_processing.rs 
	$(build_cmd) ./examples/primitives.rs 
	$(build_cmd) ./examples/primitives_scale.rs 
	$(build_cmd) ./examples/quad.rs 
	$(build_cmd) ./examples/text.rs 
	$(build_cmd) ./examples/texturing.rs 
	$(build_cmd) ./examples/window.rs 
	$(build_cmd) ./examples/wireframe.rs 
	$(build_cmd) ./examples/stereo.rs 


examples_tools:
	$(build_cmd) ./examples/recording.rs

doc:
	mkdir -p $(kiss3d_doc_path)
	rustdoc $(libs) src/lib.rs

distcheck:
	rm -rf $(tmp)
	git clone --recursive . $(tmp)
	make -C $(tmp) deps
	make -C $(tmp)
	make -C $(tmp) examples
	rm -rf $(tmp)
	git clone --recursive . $(tmp)
	make -C $(tmp) cargo
	rm -rf $(tmp)

deps:
	make lib -C $(glfw_path)
	make -C lib/nalgebra
	make deps -C lib/ncollide
	make 3df32 -C lib/ncollide
	make -C lib/gl-rs
	cd lib/rust-stb-image; ./configure
	make clean -C lib/rust-stb-image
	make -C lib/rust-stb-image
	cd lib/rust-freetype; ./configure
	make clean -C lib/rust-freetype
	make -C lib/rust-freetype

deps_recording:
	cd lib/rust-ffmpeg; ./build.sh

# manually compile ncollide and rust-fmpeg as they cannot support cargo yet.
deps_for_cargo:
	mkdir -p target/deps/
	mkdir -p target/release/deps/
	make -C lib/nalgebra
	cd lib/rust-stb-image; ./configure
	make clean -C lib/rust-stb-image
	make -C lib/rust-stb-image
	cd lib/rust-freetype; ./configure
	make clean -C lib/rust-freetype
	make -C lib/rust-freetype
	cp lib/rust-freetype/*.rlib target/deps/.
	cp lib/rust-stb-image/libstb* target/deps/.
	cp lib/rust-freetype/*.rlib target/release/deps/.
	cp lib/rust-stb-image/libstb* target/release/deps/.

cargo:
	cargo build

.PHONY:doc
.PHONY:examples
.PHONY:examples_tools
.PHONY:kiss3d
.PHONY:kiss3d_tools
