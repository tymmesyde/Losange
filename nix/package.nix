{
  lib,
  craneLib,
  revision,

  # buildInputs (build deps)
  mpv,
  libadwaita,
  libepoxy,
  openssl,

  # nativeBuildInputs (runtime deps)
  makeBinaryWrapper,
  pkg-config,
  wrapGAppsHook4,
  glib,

  # Wrapper
  nodejs,
}:
let
  # rustPlatform.buildRustPackage rebuilds every crate in Cargo.lock on any source change.
  # Splitting the build into crane's buildDepsOnly + buildPackage keeps the dependency
  # crates in their own derivation that only rebuilds when Cargo.{toml,lock} change.
  commonArgs = {
    pname = "losange";
    version = "${(builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version}-${revision}-nix";

    src = lib.cleanSource ../.;

    strictDeps = true;

    buildInputs = [
      mpv
      libadwaita
      libepoxy
      openssl
    ];

    nativeBuildInputs = [
      makeBinaryWrapper
      pkg-config
      wrapGAppsHook4
      glib
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    postInstall = ''
      install -Dm444 data/xyz.timtimtim.Losange.gschema.xml -t $out/share/gsettings-schemas/$name/glib-2.0/schemas/
      glib-compile-schemas $out/share/gsettings-schemas/$name/glib-2.0/schemas/

      install -Dm444 data/icons/xyz.timtimtim.Losange.svg -t $out/share/icons/hicolor/scalable/apps/
      install -Dm444 data/xyz.timtimtim.Losange.desktop -t $out/share/applications/
      install -Dm444 data/xyz.timtimtim.Losange.metainfo.xml -t $out/share/metainfo/

      # The application fails if '-o' is passed without an argument (e.g. when opened using a launcher)
      # therefore we match upstream's shell wrapper to handle empty URL cases.
      substituteInPlace $out/share/applications/xyz.timtimtim.Losange.desktop \
        --replace-fail "Exec=sh -c \"/usr/bin/losange -o '%u'\"" "Exec=sh -c \"losange -o '%u'\""
    '';

    # Node.js is required to run `server.js`
    # Losange will automatically download the required version of `server.js` at runtime.
    preFixup = ''
      gappsWrapperArgs+=(
        --prefix PATH : "${lib.makeBinPath [ nodejs ]}"
      )
    '';

    meta = {
      mainProgram = "losange";
      description = "Simple Stremio client for GNOME";
      homepage = "https://github.com/tymmesyde/Losange";
      license = lib.licenses.gpl3Only;
      platforms = lib.platforms.linux;
    };
  }
)
