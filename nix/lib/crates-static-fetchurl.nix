# Keep lockfile checksums while avoiding the crates.io API redirect endpoint.
fetchurl: args:
let
  matched = builtins.match "https://crates.io/api/v1/crates/([^/]+)/([^/]+)/download" (args.url or "");
in
fetchurl (
  if matched == null then args else
  let
    name = builtins.elemAt matched 0;
    version = builtins.elemAt matched 1;
  in
  args // { url = "https://static.crates.io/crates/${name}/${name}-${version}.crate"; }
)
