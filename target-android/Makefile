CMD=docker run --rm \
 	-e "TERM=xterm-256color" \
	-v "$$(pwd):/root/src" \
	-v "$$(pwd)/../engine/:/root/engine" \
	-v "$$(pwd)/../game/:/root/game" \
	-v "$$(pwd)/../util/:/root/util" \
	-w /root/src cargo-apk

build:
	# check if docker image exists, if not, build
	[ ! -z $$(docker images -q cargo-apk) ] || docker build . --tag cargo-apk
	$(CMD) cargo-apk build
run:
	[ ! -z $$(docker images -q cargo-apk) ] || docker build . --tag cargo-apk
	$(CMD) cargo-apk build
	adb uninstall xyz.noxim.kea
	adb install target/android-artifacts/app/build/outputs/apk/app-debug.apk
	adb shell monkey -p xyz.noxim.kea 1
clean:
	docker rmi cargo-apk
	# TODO: This is dumb, for some reason docker fucks up the perms in this folder
	sudo rm -rf target/
release:
	[ ! -z $$(docker images -q cargo-apk) ] || docker build . --tag cargo-apk
	$(CMD) cargo-apk build --release
	sudo jarsigner -verbose -sigalg SHA1withRSA -digestalg SHA1 -keystore release.keystore target/android-artifacts/app/build/outputs/apk/app-release-unsigned.apk kea_test
	zipalign 4 target/android-artifacts/app/build/outputs/apk/app-release-unsigned.apk app.apk

