{
  description = "Simplify checking out your Git repositories in a structured directory space.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib stdenv;

        craneLib = crane.lib.${system};
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
        ++ lib.optionals stdenv.isDarwin [pkgs.libiconv pkgs.darwin.apple_sdk.frameworks.Security pkgs.darwin.apple_sdk.frameworks.SystemConfiguration]
        ++ lib.optionals stdenv.isLinux [pkgs.dbus];

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

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          git-tool-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src buildInputs;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          };

          git-tool-doc = craneLib.cargoDoc {
            inherit cargoArtifacts src;
          };

          # Check formatting
          git-tool-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          git-tool-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

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
