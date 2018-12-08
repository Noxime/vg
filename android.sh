docker run --rm -v "$(pwd):/root/src" -w /root/src tomaka/cargo-apk cargo apk build --no-default-features --features="backend-vk"
adb install target/android-artifacts/app/build/outputs/apk/app-debug.apk
