# Rust implementation of Dungeons and Diagrams

Latest web build: [dnd.bjarkebjarke.dk](https://dnd.bjarkebjarke.dk)

For latest binaries, see the [latest release](https://github.com/bondo/dnd-rs/releases/latest).

## Compiling from source

### Desktop app

Install the rust toolchain, [bevy dependencies](https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md), [clang and lld](https://bevyengine.org/learn/quick-start/getting-started/setup/#alternative-linkers) and [cranelift](https://bevyengine.org/learn/quick-start/getting-started/setup/#cranelift for nightly-2024-08-27). Then `cargo run -p dnd-rs-desktop-app`.

### Web app

See [web app readme](https://github.com/bondo/dnd-rs/blob/main/crates/web-app/README.md)

### Android app

- Add build target: `rustup target add aarch64-linux-android`
- Install cargo-apk: `cargo install cargo-apk`
- Install keytook: `sudo apt install default-jre`
- Install Android Studio
- In `Android Studio` > `Tools` > `SDK Manager` > `Android SDK` > `SDK Tools`
  - Select `Android SDK Build-Tools`
  - Select `Android SDK Command-line Tools`
  - Select `NDK (Side by side)`
- Set environment variables

```
export ANDROID_HOME=~/Android/Sdk
# Replace the NDK version number with the version you installed
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/27.0.12077973
export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/:$PATH
```

- Run `cargo apk run -p dnd-rs-android-app --lib`
- Build `cargo apk build -p dnd-rs-android-app --lib --release`
