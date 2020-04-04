with import <nixpkgs> {}; {
  env = stdenv.mkDerivation {
    name = "rayon_adaptive";
    buildInputs = [ hwloc ];
  };
}
