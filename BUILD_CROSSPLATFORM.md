# Инструкция по сборке под Android (.apk), Linux и macOS

## 1. Сборка ядра под Android NDK (C++ / Rust staticlib)
```bash
# Установка Android NDK таргетов
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

# Сборка libzl1_core.so для Android
cargo build --target aarch64-linux-android --release -p zl1-core
```
Полученные бинарные библиотеки помещаются в Android проект в папку `app/src/main/jniLibs/arm64-v8a/libzl1_core.so`.
Заголовочный C/C++ файл доступен в `zl1-core/include/zl1.h`.

## 2. Сборка Android .APK через Tauri v2
```bash
cargo install tauri-cli --version "^2.0"
cargo tauri android init
cargo tauri android build --apk
```
*Примечание: Для локальной генерации APK требуется наличие установленного Android SDK / NDK и Java/Gradle на хосте сборщика.*

## 3. Сборка под Linux (x86_64 / aarch64)
```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

## 4. Сборка под macOS (.dmg / universal binary)
```bash
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create -output zl1-cli-universal target/x86_64-apple-darwin/release/zl1-cli target/aarch64-apple-darwin/release/zl1-cli
```
