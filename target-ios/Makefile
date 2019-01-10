build:
	cargo lipo
	cd kea && xcodebuild -target kea-debug -project kea.xcodeproj/ build
run: build
	echo "iOS port is WIP"
clean:
	echo "iOS port is WIP"
release:
	cargo lipo --release
	cd kea && xcodebuild -target kea-release -project kea.xcodeproj/ build
