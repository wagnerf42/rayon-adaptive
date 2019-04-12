with import <nixpkgs> {}; {
  env = stdenv.mkDerivation {
    name = "rayon-logs";
    buildInputs = [
      (pkgs.callPackage ./oldhwloc.nix {})
    ];
  };
}
