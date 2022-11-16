{
  description = "Simplify checking out your Git repositories in a structured directory space.";

  outputs = { self, nixpkgs }:
    {
      devShells = nixpkgs.lib.genAttrs ["aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux"] (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          lib = pkgs.lib;
          stdenv = pkgs.stdenv;
        in
        {
          default = stdenv.mkDerivation {
            name = "git-tool-shell-${system}";
            system = system;
            nativeBuildInputs = [
              pkgs.rustc
              pkgs.cargo
              pkgs.nodejs
              pkgs.protobuf
              pkgs.pkg-config
            ];

            buildInputs = [
              pkgs.libiconv
              pkgs.openssl
            ]
              ++ lib.optionals stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
              ++ lib.optionals stdenv.isLinux [ pkgs.dbus ];

            PROTOC = "${pkgs.protobuf}/bin/protoc";
          };
        }
      );

      packages = nixpkgs.lib.genAttrs ["aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux"] (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          lib = pkgs.lib;
          stdenv = pkgs.stdenv;
          rustPlatform = pkgs.rustPlatform;
          nodePackages = pkgs.nodePackages;
        in
        {
          git-tool = rustPlatform.buildRustPackage rec {
            name = "git-tool-${system}";
            pname = "git-tool";

            src = pkgs.nix-gitignore.gitignoreSourcePure ''
            /.github
            /.vscode
            /config
            /docs
            /example
            /result
            /registry
            /scripts
            /target
            *.nix
            '' ./.;

            checkFeatures = ["pure-tests"];
            
            # TODO: Enable this as soon as we can run the test suite on stable rustc
            doCheck = false;

            nativeBuildInputs = [
              pkgs.protobuf
              pkgs.pkg-config
            ];

            buildInputs = [
              pkgs.libiconv
              pkgs.openssl
            ]
              ++ lib.optionals stdenv.isDarwin [pkgs.darwin.apple_sdk.frameworks.Security]
              ++ lib.optionals stdenv.isLinux [ pkgs.dbus ];

            PROTOC = "${pkgs.protobuf}/bin/protoc";

            cargoLock = {
              lockFile = ./Cargo.lock;

              outputHashes = {
                  "tracing-0.2.0" = "sha256-xK2F6TNne+scfKgU4Vr1tfe0kdXyOZt0N7bex0Jzcmg=";
                  "mocktopus-0.7.12" = "sha256-aHaKemhKuX6HSqrbgVjpjbAs41b5OJcvPsv2kgZq6xQ=";
              };
            };

            meta = with lib; {
              description = "Simplify checking out your Git repositories in a structured directory space.";
              homepage = "https://git-tool.sierrasoftworks.com";
              license = licenses.mit;
              maintainers = [
                {
                  name = "Benjamin Pannell";
                  email = "contact@sierrasoftworks.com";
                }
              ];
            };
          };
          default = self.packages.${system}.git-tool;
        }
      );
    };
}
