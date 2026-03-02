{
  lib,
  stdenvNoCC,
  requireFile,
  nix,
}:

# Since the Apple Metal toolchain is proprietary, it cannot be distributed publicly via Nix.
# This derivation requires the user to manually import the Xcode toolchain into the Nix Store.

let
  narFile = requireFile {
    name = "apple-metal-toolchain.nar";
    sha256 = "0b56nciwx9529mqwmmh2xr42q9wv10gb28vlcqrnvjyz3snp96sl"; # AUTO_IMPORTED_HASH
    message = ''
      apple-metal-toolchain.nar not found in Nix Store.
      Please run 'just nix-import-metal-toolchain' to import your local Xcode Metal toolchain into Nix.
    '';
  };

  # Use writeScript to create the xcrun wrapper separately
  # We MUST escape ALL ${ with ''${ in Nix strings
  xcrun-wrapper = ''
    #!/usr/bin/env bash
    TOOL=""
    args=()
    SKIP_NEXT=false
    for arg in "$@"; do
        if [ "$SKIP_NEXT" = true ]; then
            SKIP_NEXT=false
            continue
        fi
        case "$arg" in
            --sdk|--find) SKIP_NEXT=true ;;
            -*) args+=("$arg") ;;
            *)
                if [ -z "$TOOL" ]; then TOOL="$arg"; else args+=("$arg"); fi
                ;;
        esac
    done
    BIN_DIR="$(cd "$(dirname "''${BASH_SOURCE[0]}")" && pwd)"
    if [ -f "$BIN_DIR/$TOOL" ]; then
        exec "$BIN_DIR/$TOOL" "''${args[@]}"
    else
        exec "$TOOL" "''${args[@]}"
    fi
  '';

in
stdenvNoCC.mkDerivation {
  pname = "apple-metal-toolchain";
  version = "1.0.0";

  dontUnpack = true;
  dontBuild = true;

  nativeBuildInputs = [ nix ];

  installPhase = ''
        set -x
        TEMP_RESTORE=$(mktemp -d)
        nix-store --restore "$TEMP_RESTORE/content" < ${narFile}
        
        mkdir -p $out
        cp -R "$TEMP_RESTORE/content/"* $out/
        
        # Create standard bin directory for PATH
        mkdir -p $out/bin
        
        link_if_exists() {
          local name=$1
          for p in "usr/bin/$name" "bin/$name"; do
            if [ -f "$out/$p" ]; then
              ln -sf "$out/$p" "$out/bin/$name"
              return 0
            fi
          done
          return 1
        }

        link_if_exists "metal" || true
        link_if_exists "metallib" || true
        link_if_exists "xcodebuild" || true
        
        mkdir -p $out/Toolchains
        ln -s $out $out/Toolchains/XcodeDefault.xctoolchain
        
        # Write the xcrun wrapper
        cat > $out/bin/xcrun <<'EOF'
    ${xcrun-wrapper}
    EOF
        chmod +x $out/bin/xcrun

        rm -rf "$TEMP_RESTORE"
        set +x
  '';

  meta = {
    description = "Apple Metal compiler toolchain for Nix sandbox builds";
    platforms = lib.platforms.darwin;
    license = lib.licenses.unfree;
  };
}
