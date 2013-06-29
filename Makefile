kiss3d_lib_path=lib
kiss3d_bin_path=bin
glfw_lib_path=glfw-rs/lib
glcore_lib_path=glcore-rs/lib
nalgebra_lib_path=nalgebra/lib
build_cmd= rust build --opt-level 3 --out-dir $(kiss3d_bin_path) -L$(kiss3d_lib_path) -L$(glfw_lib_path) -L$(glcore_lib_path) -L$(nalgebra_lib_path)

all:
	mkdir -p $(kiss3d_lib_path)
	rust build src/kiss3d.rc --opt-level 3 --out-dir $(kiss3d_lib_path) -L$(glfw_lib_path) -L$(glcore_lib_path) -L$(nalgebra_lib_path)

test:
	mkdir -p $(kiss3d_bin_path)
	$(build_cmd) src/demo/window.rs 
	$(build_cmd) src/demo/cube.rs 

deps:
	make -C glfw-rs
	make -C glcore-rs
	make -C nalgebra
