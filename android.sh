docker run --rm -v "$(pwd):/root/src" -w /root/src tomaka/cargo-apk cargo apk build
# oof for some reason our keystore is broke so lets just uninstall old ver instead of reinstalling
adb uninstall rust.kea
adb install -r target/android-artifacts/app/build/outputs/apk/app-debug.apk
