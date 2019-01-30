with import <nixpkgs> {}; {
  env = stdenv.mkDerivation {
    name = "rayon-logs";
    buildInputs = [ hwloc ];
  };
}
