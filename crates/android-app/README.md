
# Android app

## Setup

- Add build targets: `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`
- Install Java, Gradle and Kotlin: `sudo apt install default-jdk gradle kotlin`
- Install Android Studio
- In `Android Studio` > `Tools` > `SDK Manager` > `SDK Platforms` add API level 34 (Android 14.0)
- In `Android Studio` > `Tools` > `SDK Manager` > `Android SDK` > `SDK Tools`
  - Select `Android SDK Build-Tools`
  - Select `Android SDK Command-line Tools`
  - Select `NDK (Side by side)`
- Set environment variables

```
export ANDROID_HOME=~/Android/Sdk
# Replace the NDK version number with the version you installed
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/27.0.12077973
export PATH=$ANDROID_HOME/platform-tools/:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/:$PATH
```

- Install xbuild: `cargo install xbuild`
- Resolve issues identified by `x doctor`


## Run on device / emulator

- List devices with `x devices`
- Run with `x run -p dnd-rs-android-app --device [device-host]`
- Note: Seems to work with ARM only

## Create APK (for side loading)

- Install cargo-apk: `cargo install cargo-apk`
- Generate a keystore and set environment variables

```
keytool -genkeypair -keystore release.keystore -alias android -keyalg RSA -keysize 2048 -validity 10000
CARGO_APK_RELEASE_KEYSTORE=/path/to/release.keystore
CARGO_APK_RELEASE_KEYSTORE_PASSWORD=keystore password
```

- Run `cargo apk build -p dnd-rs-android-app --release --lib`
- Output: `target/release/apk/dnd.apk`
- Alternatively drop the `--release` parameter to use a debug keystore

## Create AAB (for Play Store)

- Run `x build -p dnd-rs-android-app --release --platform android --store play`
- Output: `target/x/release/android/dnd-rs-android-app.aab`
