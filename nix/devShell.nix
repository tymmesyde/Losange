{
  pkgs,
  losange,
}:
pkgs.mkShell {
  # Automatically inherit all the build dependencies (like mpv, libadwaita, pkg-config, etc.)
  # from the main package derivation so we don't have to maintain the list twice.
  inputsFrom = [ losange ];

  packages = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
    nixfmt
  ];
}
