# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

{ pkgs ? import <nixpkgs> {}}:

with builtins;
let
  inherit (pkgs) stdenv lib;

  # Tockloader v1.12.0
  tockloader = import (pkgs.fetchFromGitHub {
    owner = "tock";
    repo = "tockloader";
    rev = "v1.12.0";
    sha256 = "sha256-VgbAKDY/7ZVINDkqSHF7C0zRzVgtk8YG6O/ZmUpsh/g=";
  }) {
    inherit pkgs;
    withUnfreePkgs = false;
  };

  elf2tab = pkgs.rustPlatform.buildRustPackage rec {
    name = "elf2tab-${version}";
    version = "0.12.0";

    src = pkgs.fetchFromGitHub {
      owner = "tock";
      repo = "elf2tab";
      rev = "v${version}";
      sha256 = "sha256-+VeWLBI6md399Oaumt4pJrOkm0Nz7fmpXN2TjglUE34=";
    };

    cargoHash = "sha256-UHAwk1fBcabRqy7VMhz4aoQuIur+MQshDOhC7KFyGm4=";
  };

  rust_overlay = import "${pkgs.fetchFromGitHub {
    owner = "nix-community";
    repo = "fenix";
    rev = "4c6c7d5088f12f57afd4ba6449f9eb168ca05620";
    sha256 = "sha256-ZuVJhcL57hHBtIbaACQzlVD4p/zHOWlKh7V3wrNdnss=";
  }}/overlay.nix";

  nixpkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };

  # Get a custom cross-compile capable Rust install of a specific channel and
  # build. Tock expects a specific version of Rust with a selection of targets
  # and components to be present.
  rustBuild = (
    nixpkgs.fenix.fromToolchainFile { file = ./rust-toolchain.toml; }
  );

in
  pkgs.mkShell {
    name = "encapfn-tock-dev";

    buildInputs = with pkgs; [
      # --- Toolchains ---
      rustBuild
      openocd
      clang
      llvm
      lld
      pkgsCross.riscv32-embedded.buildPackages.gcc
      elf2tab

      # --- Convenience and support packages ---
      gnumake
      python3Full
      tockloader
      unzip # for libtock prebuilt toolchain download

      # --- CI support packages ---
      qemu

      # --- Flashing tools ---
      # If your board requires J-Link to flash and you are on NixOS,
      # add these lines to your system wide configuration.

      # Enable udev rules from segger-jlink package
      # services.udev.packages = [
      #     pkgs.segger-jlink
      # ];

      # Add "segger-jlink" to your system packages and accept the EULA:
      # nixpkgs.config.segger-jlink.acceptLicense = true;
    ];

    LD_LIBRARY_PATH="${stdenv.cc.cc.lib}/lib64:$LD_LIBRARY_PATH";
    LIBCLANG_PATH="${pkgs.libclang.lib}/lib";

    shellHook = ''
      unset LD
      unset AS
    '';
  }
