tmp=_git_distcheck
kiss3d_lib_path=lib
kiss3d_bin_path=bin
kiss3d_doc_path=doc
glfw_lib_path=lib/glfw-rs/lib
gl_lib_path=lib/gl-rs
nalgebra_lib_path=lib/nalgebra/lib
stb_image_lib_path=lib/rust-stb-image/
build_cmd= rust build --opt-level 3 --out-dir $(kiss3d_bin_path) -L$(glfw_lib_path) -L$(gl_lib_path) -L$(nalgebra_lib_path) -L$(stb_image_lib_path)

all:
	mkdir -p $(kiss3d_lib_path)
	rust build src/kiss3d.rc --opt-level 3 --out-dir $(kiss3d_lib_path) -L$(glfw_lib_path) -L$(gl_lib_path) -L$(nalgebra_lib_path) -L$(stb_image_lib_path)

test:
	mkdir -p $(kiss3d_bin_path)
	$(build_cmd) src/demo/lines.rs 
	$(build_cmd) src/demo/cube.rs 
	$(build_cmd) src/demo/camera.rs 
	$(build_cmd) src/demo/window.rs 
	$(build_cmd) src/demo/event.rs 
	$(build_cmd) src/demo/quad.rs 
	$(build_cmd) src/demo/primitives.rs 
	$(build_cmd) src/demo/primitives_scale.rs 
	$(build_cmd) src/demo/texturing.rs 

doc:
	mkdir -p $(kiss3d_doc_path)
	rust doc src/kiss3d.rc --output-dir $(kiss3d_doc_path)

distcheck:
	rm -rf $(tmp)
	git clone --recursive . $(tmp)
	make -C $(tmp) deps
	make -C $(tmp)
	make -C $(tmp) test
	rm -rf $(tmp)

deps:
	make -C lib/glfw-rs
	make -C lib/nalgebra
	cd lib/gl-rs; rustc --opt-level=3 gl_ptr.rs
	cd lib/rust-stb-image; ./configure
	make clean -C lib/rust-stb-image
	make -C lib/rust-stb-image

.PHONY:doc
