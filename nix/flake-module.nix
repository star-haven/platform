/**
  Copyright (c) 2023 Sridhar Ratnakumar

  Permission is hereby granted, free of charge, to any person obtaining a copy
  of this software and associated documentation files (the "Software"), to deal
  in the Software without restriction, including without limitation the rights
  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
  copies of the Software, and to permit persons to whom the Software is
  furnished to do so, subject to the following conditions:

  The above copyright notice and this permission notice shall be included in all
  copies or substantial portions of the Software.

  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
  SOFTWARE.
 */
{ self, lib, inputs, flake-parts-lib, ... }:

let
  inherit (flake-parts-lib)
    mkPerSystemOption;
in
{
  options = {
    perSystem = mkPerSystemOption
      ({ config, self', inputs', pkgs, system, ... }: {
        options = {
          leptos-fullstack.overrideCraneArgs = lib.mkOption {
            type = lib.types.functionTo lib.types.attrs;
            default = _: { };
            description = "Override crane args for the leptos-fullstack package";
          };

          leptos-fullstack.rustToolchain = lib.mkOption {
            type = lib.types.package;
            description = "Rust toolchain to use for the leptos-fullstack package";
            default = (pkgs.rust-bin.fromRustupToolchainFile (self + /rust-toolchain.toml)).override {
              extensions = [
                "rust-src"
                "rust-analyzer"
                "clippy"
                "rustc-codegen-cranelift-preview"
              ];
            };
          };

          leptos-fullstack.craneLib = lib.mkOption {
            type = lib.types.lazyAttrsOf lib.types.raw;
            default = (inputs.crane.mkLib pkgs).overrideToolchain config.leptos-fullstack.rustToolchain;
          };

          leptos-fullstack.src = lib.mkOption {
            type = lib.types.path;
            description = "Source directory for the leptos-fullstack package";
            # When filtering sources, we want to allow assets other than .rs files
            # TODO: Don't hardcode these!
            default = lib.cleanSourceWith {
              src = self; # The original, unfiltered source
              filter = path: type:
                (lib.hasSuffix "\.html" path) ||
                (lib.hasSuffix "tailwind.config.js" path) ||
                # Example of a folder for images, icons, etc
                (lib.hasInfix "/assets/" path) ||
                (lib.hasInfix "/css/" path) ||
                # Default filter from crane (allow .rs files)
                (config.leptos-fullstack.craneLib.filterCargoSources path type)
              ;
            };
          };
        };
        config =
          let
            cargoToml = builtins.fromTOML (builtins.readFile (self + /Cargo.toml));
            inherit (cargoToml.package) name version;
            inherit (config.leptos-fullstack) rustToolchain craneLib src;

            # Crane builder for cargo-leptos projects
            craneBuild = rec {
              args = {
                inherit src;
                pname = name;
                version = version;
                buildInputs = [
                  pkgs.cargo-leptos
                  pkgs.binaryen # Provides wasm-opt
                  tailwindcss
                ];
              };
              cargoArtifacts = craneLib.buildDepsOnly args;
              buildArgs = args // {
                inherit cargoArtifacts;
                buildPhaseCargoCommand = "cargo leptos build --release -vvv";
                cargoTestCommand = "cargo leptos test --release -vvv";
                nativeBuildInputs = [
                  pkgs.makeWrapper
                ];
                installPhaseCommand = ''
                  mkdir -p $out/bin
                  cp target/server/release/${name} $out/bin/
                  cp -r target/site $out/bin/
                  wrapProgram $out/bin/${name} \
                    --set LEPTOS_SITE_ROOT $out/bin/site
                '';
              };
              package = craneLib.buildPackage (buildArgs // config.leptos-fullstack.overrideCraneArgs buildArgs);

              check = craneLib.cargoClippy (args // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets --all-features -- --deny warnings";
              });

              doc = craneLib.cargoDoc (args // {
                inherit cargoArtifacts;
              });
            };

            rustDevShell = pkgs.mkShell.override {
              stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
            } {
              shellHook = ''
                # For rust-analyzer 'hover' tooltips to work.
                export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library";
              '';
              buildInputs = [
                pkgs.libiconv
              ];
              nativeBuildInputs = [
                rustToolchain
              ];
            };

            tailwindcss = pkgs.nodePackages.tailwindcss.overrideAttrs
              (oa: {
                plugins = [
                  pkgs.nodePackages."@tailwindcss/aspect-ratio"
                  pkgs.nodePackages."@tailwindcss/forms"
                  pkgs.nodePackages."@tailwindcss/language-server"
                  pkgs.nodePackages."@tailwindcss/line-clamp"
                  pkgs.nodePackages."@tailwindcss/typography"
                ];
              });
          in
          {
            # Rust package
            packages.${name} = craneBuild.package;
            packages."${name}-doc" = craneBuild.doc;

            checks."${name}-clippy" = craneBuild.check;

            # Rust dev environment
            devShells.${name} = pkgs.mkShell {
              inputsFrom = [
                rustDevShell
              ];
              nativeBuildInputs = with pkgs; [
                tailwindcss
                cargo-leptos
                binaryen # Provides wasm-opt
              ];
            };
          };
      });
  };
}
