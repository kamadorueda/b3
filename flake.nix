# SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
#
# SPDX-License-Identifier: AGPL-3.0-only
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs: let
    commit = inputs.self.shortRev or "dirty";
    date = inputs.self.lastModifiedDate or inputs.self.lastModified or "19700101";
    version = "0.1.0+${builtins.substring 0 8 date}.${commit}";

    nixpkgsForHost = host:
      import inputs.nixpkgs {
        overlays = [
          (self: super: {
            toros = self.rustPlatform.buildRustPackage {
              pname = "toros";
              inherit version;
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

              meta = {
                description = "";
                homepage = "https://github.com/kamadorueda/toros";
                license = self.lib.licenses.agpl3Only;
                maintainers = [self.lib.maintainers.kamadorueda];
                platforms = self.lib.systems.doubles.all;
              };
            };
          })
        ];
        system = host;
      };

    nixpkgs."aarch64-darwin" = nixpkgsForHost "aarch64-darwin";
    nixpkgs."aarch64-linux" = nixpkgsForHost "aarch64-linux";
    nixpkgs."i686-linux" = nixpkgsForHost "i686-linux";
    nixpkgs."x86_64-darwin" = nixpkgsForHost "x86_64-darwin";
    nixpkgs."x86_64-linux" = nixpkgsForHost "x86_64-linux";
  in {
    devShells."x86_64-linux".default = with nixpkgs."x86_64-linux";
      mkShell {
        name = "toros";
        packages = [
          cargo
          cargo-tarpaulin
          clippy
          entr
          jq
          linuxPackages_latest.perf
          reuse
          rustc
        ];
      };

    apps."x86_64-linux".dev = with nixpkgs."x86_64-linux"; {
      type = "app";
      program =
        (writeShellScript "license" ''
          git ls-files | entr sh -euc '
            UPDATE=1 cargo test
            cargo doc
            cargo tarpaulin -o html
          '
        '')
        .outPath;
    };

    apps."x86_64-linux".license = with nixpkgs."x86_64-linux"; {
      type = "app";
      program =
        (writeShellScript "license" ''
          copyright='Kevin Amado <kamadorueda@gmail.com>'
          license='AGPL-3.0-only'

          git ls-files | xargs reuse addheader \
            --copyright="$copyright" \
            --license="$license" \
            --skip-unrecognised
        '')
        .outPath;
    };

    packages."aarch64-darwin".default = nixpkgs."aarch64-darwin".toros;
    packages."aarch64-linux".default = nixpkgs."aarch64-linux".toros;
    packages."i686-linux".default = nixpkgs."i686-linux".toros;
    packages."x86_64-darwin".default = nixpkgs."x86_64-darwin".toros;
    packages."x86_64-linux".default = nixpkgs."x86_64-linux".toros;
  };
}
