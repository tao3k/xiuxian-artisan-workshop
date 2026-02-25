{
  pkgs,
  config,
  __inputs__,
  ...
}:
let
  inherit (__inputs__) nixpkgs-latest;
  # Darwin-specific build reliability note:
  # `uv sync --reinstall-package omni-core-rs` triggers maturin -> cargo in a subprocess.
  # With the default Nix-provided cargo script, some subprocess environments hit:
  # `dyld: Symbol not found: _libiconv` (via libidn2/libiconv lookup mismatch).
  # This shim keeps a stable `/usr/bin/env bash` entrypoint and delegates to the
  # toolchain cargo binary, which avoids that dynamic-link failure in practice.
  cargoShim = pkgs.writeTextFile {
    name = "cargo";
    destination = "/bin/cargo";
    executable = true;
    text = ''
      #!/usr/bin/env bash
      # Keep a stable cargo entrypoint for uv/maturin subprocess invocation on Darwin.
      exec ${config.languages.rust.toolchainPackage}/bin/cargo "$@"
    '';
  };
in
{
  packages = [
    cargoShim
    pkgs.protobuf
    pkgs.openssl
    pkgs.pkg-config
    pkgs.libidn2
    pkgs.cargo-nextest
    pkgs.cargo-audit
    pkgs.cargo-deny
  ];
  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "stable";
    # Ensure rust can link python library
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  env = {
    PYO3_PYTHON = "${config.languages.python.package}/bin/python";
    PROTOC = "${pkgs.protobuf}/bin/protoc";
    # Fix PyO3 extension module linking for cargo test
    # Add Python library path for macOS and Linux
    # PYTHON_LIB_PATH = "${config.languages.python.package}/lib";
    # DYLD_LIBRARY_PATH = "${config.languages.python.package}/lib:${pkgs.openssl.out}/lib:''\${DYLD_LIBRARY_PATH:-}";
    # LD_LIBRARY_PATH = "${config.languages.python.package}/lib:${pkgs.openssl.out}/lib:''\${LD_LIBRARY_PATH:-}";
  };
}
