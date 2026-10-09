# SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
#
# SPDX-License-Identifier: AGPL-3.0-only

{
  inputs = {
    blank.url = "github:divnix/blank";

    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    fenix.inputs.rust-analyzer-src.follows = "blank";

    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs: let
    commit = inputs.self.shortRev or "dirty";
    date = inputs.self.lastModifiedDate or inputs.self.lastModified or "19700101";
    version = "0.1.0+${builtins.substring 0 8 date}.${commit}";

    overlay = self: super: {
      b3 = self.rustPlatform.buildRustPackage {
        pname = "b3";
        inherit version;
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        # nativeBuildInputs = [
        #   self.pkg-config
        #   self.pkgsBuildBuild.rustPlatform.bindgenHook
        # ];
        # buildInputs = [self.gcc.cc.lib];

        # buildType = "debug";
        # doCheck = false;

        meta = {
          description = "Parser for the Nix Expressions Language";
          homepage = "https://github.com/kamadorueda/b3";
          license = self.lib.licenses.agpl3Only;
          maintainers = [self.lib.maintainers.kamadorueda];
          platforms = self.lib.systems.doubles.all;
        };
      };
    };

    nixpkgsFor = system:
      import inputs.nixpkgs {
        overlays = [overlay];
        inherit system;
      };

    outputsFor = system: let
      fenix = inputs.fenix.packages.${system};
      nixpkgs = nixpkgsFor system;
    in {
      packages.${system}.default = nixpkgs.b3;

      devShells.${system}.default = nixpkgs.mkShell {
        name = "b3";
        packages = [
          fenix.stable.toolchain
          nixpkgs.cargo-license
          nixpkgs.cargo-tarpaulin
          nixpkgs.cargo-watch
          nixpkgs.clang
          nixpkgs.linuxPackages_latest.perf
          nixpkgs.mdbook
          nixpkgs.mdbook-linkcheck
          nixpkgs.reuse
          nixpkgs.valgrind

          # License stuff
          (nixpkgs.writeShellApplication {
            name = "license";

            runtimeInputs = [nixpkgs.reuse];

            text = ''
              copyright='Kevin Amado <kamadorueda@gmail.com>'
              license='AGPL-3.0-only'

              find . -type f -name '*.license' -exec rm {} +

              reuse addheader \
                --copyright="$copyright" \
                --license="$license" \
                --force-dot-license \
                Cargo.lock \
                flake.lock \

              git ls-files \
                | xargs reuse addheader \
                  --copyright="$copyright" \
                  --license="$license" \
                  --skip-unrecognised

              reuse lint
            '';
          })
        ];
        shellHook = ''
          export LIBCLANG_PATH="${nixpkgs.libclang.lib}/lib"
        '';
      };
    };
  in
    builtins.foldl' inputs.nixpkgs.lib.recursiveUpdate {} [
      (outputsFor "aarch64-darwin")
      (outputsFor "aarch64-linux")
      (outputsFor "x86_64-darwin")
      (outputsFor "x86_64-linux")
      {
        packages."x86_64-linux".static = (nixpkgsFor "x86_64-linux").pkgsStatic.b3;
      }
    ];
}
