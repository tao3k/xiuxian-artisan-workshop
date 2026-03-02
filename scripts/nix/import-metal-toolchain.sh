#!/usr/bin/env bash
set -euo pipefail

# This script automates the extraction and import of the Apple Metal toolchain into the Nix store.
# Now simplified to only handle the toolchain, as we use native Nixpkgs for the SDK.

NAR_FILE="apple-metal-toolchain.nar"
NIX_PKG_FILE="nix/packages/apple-metal-toolchain.nix"
MOUNT_POINT="/private/tmp/metal-dmg-import"

echo "🔍 Searching for Metal Toolchain system asset..."
DMG_PATH=$(find /System/Library/AssetsV2/com_apple_MobileAsset_MetalToolchain -name "*.dmg" | head -1)

if [ -z "$DMG_PATH" ]; then
  echo "❌ Error: Metal Toolchain DMG not found. Run: xcodebuild -downloadComponent MetalToolchain"
  exit 1
fi

cleanup() {
  [ -d "$MOUNT_POINT" ] && (
    hdiutil detach "$MOUNT_POINT" -quiet || true
    rmdir "$MOUNT_POINT" || true
  )
  rm -f "$NAR_FILE"
}
trap cleanup EXIT

mkdir -p "$MOUNT_POINT"
echo "📥 Attaching Metal Toolchain DMG..."
hdiutil attach "$DMG_PATH" -mountpoint "$MOUNT_POINT" -readonly -quiet

echo "📦 Dumping Metal toolchain to $NAR_FILE..."
nix-store --dump "$MOUNT_POINT/Metal.xctoolchain" >"$(pwd)/$NAR_FILE"

echo "📥 Adding toolchain to Nix Store..."
STORE_PATH=$(nix-store --add-fixed sha256 "$NAR_FILE")
REAL_HASH=$(nix-hash --type sha256 --flat --base32 "$NAR_FILE")

echo "✅ Toolchain added to store: $STORE_PATH"
echo "🔑 Base32 Hash: $REAL_HASH"

if [ ! -f "$NIX_PKG_FILE" ]; then
  echo "❌ Error: $NIX_PKG_FILE not found."
  exit 1
fi

echo "📝 Updating $NIX_PKG_FILE with new hash..."
# Portable replacement using temp file
sed "s/sha256 = \".*\"; # \(DEFAULT_PLACEHOLDER\|AUTO_IMPORTED_HASH\)/sha256 = \"$REAL_HASH\"; # AUTO_IMPORTED_HASH/g" "$NIX_PKG_FILE" >"$NIX_PKG_FILE.tmp"
mv "$NIX_PKG_FILE.tmp" "$NIX_PKG_FILE"

echo ""
echo "🎉 Success! The Metal toolchain has been imported and Nix native SDK is configured."
echo "   You can now build xiuxian-llm:"
echo "   just nix-build-xiuxian-llm"
