#!/usr/bin/env bash
# Builds the cdylib with cargo-ndk and assembles a signed APK by hand:
# aapt2 link (manifest only, no resources) + the .so + zipalign + apksigner.
# No Gradle — a NativeActivity app with hasCode=false needs none of it.
#
# Usage: ./build_apk.sh [--release]
# Produces: target/apk/kiss3d-android-example.apk
set -euo pipefail
cd "$(dirname "$0")"

SDK=${ANDROID_HOME:-$HOME/Library/Android/sdk}
BT="$SDK/build-tools/35.0.1"
export ANDROID_NDK_HOME=${ANDROID_NDK_HOME:-$SDK/ndk/android-ndk-r27d}

PROFILE_FLAG=${1:-}
PROFILE_DIR=debug
if [ "$PROFILE_FLAG" = "--release" ]; then PROFILE_DIR=release; fi

cargo ndk --platform 26 -t arm64-v8a -o target/jniLibs build ${PROFILE_FLAG:+--release}

# The debug keystore Android tooling normally creates on first use; make one
# if this machine has never built an Android app the Gradle way.
if [ ! -f "$HOME/.android/debug.keystore" ]; then
  mkdir -p "$HOME/.android"
  keytool -genkeypair -keystore "$HOME/.android/debug.keystore" \
    -storepass android -keypass android -alias androiddebugkey \
    -dname "CN=Android Debug,O=Android,C=US" -keyalg RSA -validity 10000
fi

rm -rf target/apk && mkdir -p target/apk/lib/arm64-v8a
cp target/jniLibs/arm64-v8a/*.so target/apk/lib/arm64-v8a/

"$BT/aapt2" link -o target/apk/base.apk \
  --manifest AndroidManifest.xml \
  -I "$SDK/platforms/android-35/android.jar"

(cd target/apk && zip -q -u base.apk lib/arm64-v8a/*.so)
"$BT/zipalign" -f 4 target/apk/base.apk target/apk/aligned.apk
"$BT/apksigner" sign \
  --ks "$HOME/.android/debug.keystore" --ks-pass pass:android --key-pass pass:android \
  --out target/apk/kiss3d-android-example.apk target/apk/aligned.apk

echo "APK: $(pwd)/target/apk/kiss3d-android-example.apk"
