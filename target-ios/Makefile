build:
	cargo lipo
	cd vg && xcodebuild -target vg-debug -project vg.xcodeproj/ build
run: build
	echo "iOS port is WIP"
clean:
	echo "iOS port is WIP"
release:
	cargo lipo --release
	cd vg && xcodebuild -target vg-release -project vg.xcodeproj/ build
