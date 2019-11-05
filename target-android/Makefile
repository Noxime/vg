CMD=docker run --rm \
 	-e "TERM=xterm-256color" \
	-v "$$(pwd)/../..:/root/" \
	-v "$$(pwd)/../engine/:/root/vg/engine" \
	-v "$$(pwd)/../util/:/root/vg/util" \
	-w /root/vg/target-android/src cargo-apk

build:
	# check if docker image exists, if not, build
	# [ ! -z $$(docker images -q cargo-apk) ] || docker build . --tag cargo-apk
	$(CMD) cargo apk build

run:
	# [ ! -z $$(docker images -q cargo-apk) ] || docker build . --tag cargo-apk
	$(CMD) cargo-apk build
	# adb uninstall xyz.noxim.vg
	adb install target/android-artifacts/debug/apk/target-android.apk
	adb shell monkey -p xyz.noxim.vg 1
	adb logcat -c
	adb logcat -v color | grep Rust

clean:
	docker rmi cargo-apk
	# TODO: This is dumb, for some reason docker fucks up the perms in this folder
	sudo rm -rf target/

release:
	[ ! -z $$(docker images -q cargo-apk) ] || docker build . --tag cargo-apk
	$(CMD) cargo-apk build --release
	sudo jarsigner -verbose -sigalg SHA1withRSA -digestalg SHA1 -keystore release.keystore target/android-artifacts/app/build/outputs/apk/app-release-unsigned.apk vg_test
	zipalign 4 target/android-artifacts/app/build/outputs/apk/app-release-unsigned.apk app.apk

