tmp=_git_distcheck

all:
	cargo build --release

test: examples

examples:
	cd examples; cargo build --release

doc:
	cargo doc

distcheck:
	rm -rf $(tmp)
	git clone . $(tmp)
	make -C $(tmp)
	make examples -C $(tmp)
	rm -rf $(tmp)

clean:
	cargo clean
