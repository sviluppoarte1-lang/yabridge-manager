.PHONY: build deb appimage clean

build:
	cargo build --release

deb: build
	bash packaging/deb/build.sh

appimage: build
	bash packaging/appimage/build.sh

clean:
	cargo clean
	rm -rf build/ *.deb *.AppImage
