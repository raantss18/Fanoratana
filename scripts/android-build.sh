#!/usr/bin/env bash
# Construit l'APK de Fanoratana : la bibliothèque Rust (cargo-ndk), puis Gradle.
#   scripts/android-build.sh [--debug]
# Sortie : dist/fanoratana-<version>-arm64-v8a.apk
set -euo pipefail
cd "$(dirname "$0")/.."
RACINE=$PWD
NDK_VERSION=$(sed -n 's/.*ndkVersion "\(.*\)"/\1/p' android/app/build.gradle)
for sdk in "${ANDROID_HOME:-}" "${ANDROID_SDK_ROOT:-}" "$HOME/Android/Sdk"; do
  if [ -n "$sdk" ] && [ -d "$sdk/ndk/$NDK_VERSION" ]; then export ANDROID_HOME=$sdk; break; fi
done
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/$NDK_VERSION
echo "sdk.dir=$ANDROID_HOME" > android/local.properties
# Le cargo de rustup (avec les cibles Android) avant celui de la distribution.
[ -x "$HOME/.cargo/bin/cargo" ] && export PATH="$HOME/.cargo/bin:$PATH"
profil=android; type=Release
[ "${1:-}" = --debug ] && { profil=dev; type=Debug; }
jni="$RACINE/android/app/src/main/jniLibs"
rm -rf "$jni"
(cd apps/fanoratana-android && cargo ndk -t arm64-v8a -P 26 -o "$jni" build -p fanoratana-android --profile "$profil" --target-dir "$RACINE/target/android")
find "$jni" -name '*.so' ! -name 'libfanoratana_android.so' -delete
(cd android && ./gradlew --quiet "assemble$type")
version=$(cat VERSION)
mkdir -p dist
bas=$(tr '[:upper:]' '[:lower:]' <<< "$type")
cp "android/app/build/outputs/apk/$bas/"*.apk "dist/fanoratana-$version-arm64-v8a.apk"
ls -la dist/
