.PHONY: build deb appimage clean

build:
	cargo build --release

deb: build
	bash packaging/deb/build.sh

appimage: build
	APPIMAGE_EXTRACT_AND_RUN=1 bash packaging/appimage/build.sh

clean:
	cargo clean
	rm -rf build/ *.deb *.AppImage
