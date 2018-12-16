CMD=docker run --rm -v "$$(pwd):/root/src" -v "$$(pwd)/../engine/:/root/engine" -v "$$(pwd)/../game/:/root/game" -w /root/src tomaka/cargo-apk

build:
	$(CMD) cargo apk build
run:
	$(CMD) cargo apk build
	adb install target/android-artifacts/app/build/outputs/apk/app-debug.apk
	adb shell monkey -p xyz.noxim.kea 1
clean:
	# TODO: This is dumb, for some reason docker fucks up the perms in this folder
	sudo rm -rf target/
release:
	$(CMD)cargo apk build --release