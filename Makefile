tmp=_git_distcheck
kiss3d_lib_path=lib
kiss3d_bin_path=bin
kiss3d_doc_path=doc
glfw_lib_path=lib/glfw-rs
gl_lib_path=lib/gl-rs
nalgebra_lib_path=lib/nalgebra/lib
stb_image_lib_path=lib/rust-stb-image/
build_cmd= rust build --opt-level 3 --out-dir $(kiss3d_bin_path) -L$(glfw_lib_path) -L$(gl_lib_path) -L$(nalgebra_lib_path) -L$(stb_image_lib_path)

all:
	mkdir -p $(kiss3d_lib_path)
	rust build src/lib.rs --opt-level 3 --out-dir $(kiss3d_lib_path) -L$(glfw_lib_path) -L$(gl_lib_path) -L$(nalgebra_lib_path) -L$(stb_image_lib_path)

test: examples


examples:
	mkdir -p $(kiss3d_bin_path)
	$(build_cmd) ./examples/camera.rs 
	$(build_cmd) ./examples/wireframe.rs 
	$(build_cmd) ./examples/cube.rs 
	$(build_cmd) ./examples/primitives.rs 
	$(build_cmd) ./examples/texturing.rs 
	$(build_cmd) ./examples/lines.rs 
	$(build_cmd) ./examples/window.rs 
	$(build_cmd) ./examples/event.rs 
	$(build_cmd) ./examples/quad.rs 
	$(build_cmd) ./examples/primitives_scale.rs 
	$(build_cmd) ./examples/post_processing.rs 
	$(build_cmd) ./examples/add_remove.rs 

doc:
	mkdir -p $(kiss3d_doc_path)
	rust doc src/lib.rs --output-dir $(kiss3d_doc_path)

distcheck:
	rm -rf $(tmp)
	git clone --recursive . $(tmp)
	make -C $(tmp) deps
	make -C $(tmp)
	make -C $(tmp) examples
	rm -rf $(tmp)

deps:
	rustc --out-dir $(glfw_lib_path) --opt-level=3 lib/glfw-rs/src/glfw/lib.rs
	make -C lib/nalgebra
	cd lib/gl-rs; rustc --opt-level=3 gl.rs
	cd lib/rust-stb-image; ./configure
	make clean -C lib/rust-stb-image
	make -C lib/rust-stb-image

.PHONY:doc
.PHONY:examples
