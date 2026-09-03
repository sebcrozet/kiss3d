#!/usr/bin/env bash
# Builds the example for the iOS simulator and wraps it in a minimal .app
# bundle. The simulator runs unsigned bundles, so no Xcode project and no
# signing identity are needed:
#
#   ./build_app.sh
#   xcrun simctl boot "iPhone 16"          # once
#   xcrun simctl install booted target/kiss3d.app
#   xcrun simctl launch --console booted rs.kiss3d.example
#
# For a real device, build --target aarch64-apple-ios instead and sign the
# bundle with your own identity; the bundle layout is the same.
set -euo pipefail
cd "$(dirname "$0")"

TARGET=${1:-aarch64-apple-ios-sim}

cargo build --target "$TARGET"

APP=target/kiss3d.app
rm -rf "$APP" && mkdir -p "$APP"
cp "target/$TARGET/debug/kiss3d-ios-example" "$APP/"
cp Info.plist "$APP/"

echo "APP: $(pwd)/$APP"
