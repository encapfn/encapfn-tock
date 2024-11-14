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

  rust_overlay = import "${pkgs.fetchFromGitHub {
    owner = "nix-community";
    repo = "fenix";
    rev = "1a92c6d75963fd594116913c23041da48ed9e020";
    sha256 = "sha256-L3vZfifHmog7sJvzXk8qiKISkpyltb+GaThqMJ7PU9Y=";
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

      # --- Convenience and support packages ---
      just
      python3Full
      tockloader

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
