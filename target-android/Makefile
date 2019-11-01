CMD=docker run --rm \
 	-e "TERM=xterm-256color" \
	-v "$$(pwd)/../..:/root/" \
	-v "$$(pwd)/../engine/:/root/vg/engine" \
	-v "$$(pwd)/../util/:/root/vg/util" \
	-w /root/vg/src cargo-apk

# x:
# 	$(CMD) curl https://sh.rustup.rs -sSf | sh -s -- -y \
# 	&& bash /home/noxim/.cargo/env \
# 	&& ANDROID_HOME=/opt/android-sdk-linux NDK_HOME=/usr/local/android-ndk-r15b cargo apk build

build:
	rustup target add armv7-linux-androideabi
	rustup target add aarch64-linux-android
	rustup target add i686-linux-android
	rustup target add x86_64-linux-android
	PATH=$$PATH:~/Android/Sdk/platform-tools:~/Android/Sdk/tools NDK_HOME=~/Android/Sdk/ndk/20.0.5594570/ ANDROID_HOME=~/Android/Sdk/ cargo apk build

build-docker:
	# check if docker image exists, if not, build
	[ ! -z $$(docker images -q cargo-apk) ] || docker build . --tag cargo-apk
	$(CMD) cargo apk build
run-docker:
	[ ! -z $$(docker images -q cargo-apk) ] || docker build . --tag cargo-apk
	$(CMD) cargo-apk build
	adb uninstall xyz.noxim.vg
	adb install target/android-artifacts/app/build/outputs/apk/app-debug.apk
	adb shell monkey -p xyz.noxim.vg 1
clean:
	docker rmi cargo-apk
	# TODO: This is dumb, for some reason docker fucks up the perms in this folder
	sudo rm -rf target/
release-docker:
	[ ! -z $$(docker images -q cargo-apk) ] || docker build . --tag cargo-apk
	$(CMD) cargo-apk build --release
	sudo jarsigner -verbose -sigalg SHA1withRSA -digestalg SHA1 -keystore release.keystore target/android-artifacts/app/build/outputs/apk/app-release-unsigned.apk vg_test
	zipalign 4 target/android-artifacts/app/build/outputs/apk/app-release-unsigned.apk app.apk

