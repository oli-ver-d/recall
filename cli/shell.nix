{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.openssl
    pkgs.pkg-config
    pkgs.libgcc
    pkgs.gcc_multi
    pkgs.gcc9
    pkgs.ocl-icd
    pkgs.opencl-headers
  ];
}
