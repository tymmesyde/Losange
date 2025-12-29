#!/bin/sh

app_id="xyz.timtimtim.Losange"
cwd="build-aux/flatpak"

python3 $cwd/flatpak-builder-tools/cargo/flatpak-cargo-generator.py Cargo.lock -o $cwd/cargo-sources.json

sed 's/usr/app/g' data/$app_id.desktop > $cwd/$app_id.desktop

flatpak-builder --repo=$cwd/repo --force-clean $cwd/build $cwd/$app_id.json
flatpak build-bundle $cwd/repo $cwd/$app_id.flatpak $app_id