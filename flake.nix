{
  description = "Simplify checking out your Git repositories in a structured directory space.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib stdenv;

        craneLib = crane.mkLib pkgs;
        src = let
          # Only keeps markdown files
          testDataFilter = path: _type: builtins.match ".*/src/test/.*$" path != null;
          registryDataFilter = path: _type: builtins.match ".*/registry/.*$" path != null;
          testDataOrCargo = path: type:
            (testDataFilter path type) || (registryDataFilter path type) || (craneLib.filterCargoSources path type);
        in
        lib.cleanSourceWith {
          src = ./.;
          filter = testDataOrCargo;
        };

        nativeBuildInputs = [pkgs.pkg-config pkgs.protobuf pkgs.gitMinimal];

        buildInputs = [ pkgs.openssl ]
        ++ lib.optionals stdenv.hostPlatform.isDarwin [pkgs.libiconv]
        ++ lib.optionals stdenv.hostPlatform.isLinux [pkgs.dbus];

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src nativeBuildInputs buildInputs;
        };

        git-tool = craneLib.buildPackage {
          inherit cargoArtifacts src nativeBuildInputs buildInputs;

          doCheck = false;
        };
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit git-tool;

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `my-crate` if you do not want
          # the tests to run twice
          git-tool-nextest = craneLib.cargoNextest {
            inherit cargoArtifacts src nativeBuildInputs buildInputs;
            partitions = 1;
            partitionType = "count";

            # Disable impure tests (which access the network and/or filesystem)
            cargoNextestExtraArgs = "--no-fail-fast --features default,pure-tests";
          };
        };

        packages.default = git-tool;

        apps.default = flake-utils.lib.mkApp {
          drv = git-tool;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          # Extra inputs can be added here
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            nodejs
          ] ++ nativeBuildInputs ++ buildInputs;
        };
      });
}
