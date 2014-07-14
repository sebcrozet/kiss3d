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

all:
	cargo build -u --release

examples_tools:
	$(build_cmd) ./examples/recording.rs

doc:
	mkdir -p $(kiss3d_doc_path)
	rustdoc $(libs) src/lib.rs

distcheck:
	rm -rf $(tmp)
	git clone . $(tmp)
	make -C $(tmp)
	rm -rf $(tmp)

.PHONY:doc
