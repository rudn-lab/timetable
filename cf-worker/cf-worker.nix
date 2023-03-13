# https://ryantm.github.io/nixpkgs/builders/special/fhs-environments/
{ pkgs ? import <nixpkgs> {} }:
(pkgs.buildFHSUserEnv {
  name = "cf-worker";
  targetPkgs = pkgs: (with pkgs; [
    nodePackages.npm
    nodePackages.wrangler
    worker-build
    wasm-pack
    pkg-config
    openssl.dev
  ]);
  runScript = "fish";
}).env
