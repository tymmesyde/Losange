#!/bin/sh

app_id="xyz.timtimtim.Losange"

python3 flatpak/flatpak-builder-tools/cargo/flatpak-cargo-generator.py Cargo.lock -o flatpak/cargo-sources.json

sed 's/usr/app/g' data/$app_id.desktop > flatpak/$app_id.desktop

flatpak-builder --force-clean flatpak/build $app_id.json
flatpak build-export flatpak/repo flatpak/build
flatpak build-bundle flatpak/repo flatpak/$app_id.flatpak $app_id