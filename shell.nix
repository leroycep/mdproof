{ pkgs ? import <nixpkgs> {} }:
with pkgs;

stdenv.mkDerivation {
name = "mdproof-env";
buildInputs = [
  rustChannels.stable.rust
  gcc
];
}
