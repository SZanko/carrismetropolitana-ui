{
  description = "Cross-platform template for Slint and Rust apps using Nix";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    #nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self
    , nixpkgs
    , rust-overlay
    , flake-utils
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
          android_sdk.accept_license = if roles.android then true else false;
        };

        isMac = pkgs.stdenv.isDarwin;
        isLinux = pkgs.stdenv.isLinux;

        # check if a flag is enabled
        enabled = v: v == "1";

        roles = {
          android = if enabled (builtins.getEnv "NO_ANDROID") then false else true;
          ios =
            if enabled (builtins.getEnv "NO_IOS") then
              false
              # otherwise (if NO_IOS is not set):
            else if isMac then
              true # iOS utils only work on macOS ...
            else
              false; # ... and not on Linux (or other future platforms)
          linux =
            if enabled (builtins.getEnv "NO_LINUX") then
              false
              # otherwise (if NO_LINUX is not set):
            else if isLinux then
              true # Linux utils only work on a Linux machine...
            else
              false; # ... and not on macOS (or other future platforms)
          macos =
            if enabled (builtins.getEnv "NO_MACOS") then
              false
              # otherwise (if NO_MACOS is not set):
            else if isMac then
              true # macOS utils only work on a macOS machine ...
            else
              false; # ... and not on Linux (or other future platforms)
        };

        # minimal set of packages
        rustTools = with pkgs; [
          (rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
            targets =
              # Linux targets
              (
                if roles.linux then
                  if system == "aarch64-linux" then
                    [ "aarch64-unknown-linux-gnu" ]
                  else if system == "x86_64-linux" then
                    [ "x86_64-unknown-linux-gnu" ]
                  else
                    builtins.throw "Unsupported Linux architecture: ${system}. Only aarch64-linux and x86_64-linux are supported."
                else
                  [ ]
              )
              ++
              # MacOS targets
              (
                if roles.macos then
                  if system == "aarch64-darwin" then
                    [ "aarch64-apple-darwin" ]
                  else if system == "x86_64-darwin" then
                    [ "x86_64-apple-darwin" ]
                  else
                    builtins.throw "Unsupported macOS architecture: ${system}. Only aarch64-darwin and x86_64-darwin are supported."
                else
                  [ ]
              )
              ++
              # Android targets
              (
                if roles.android then
                  [
                    "aarch64-linux-android"
                    "x86_64-linux-android"
                    # optionally include older "armv7-linux-androideabi" and "i686-linux-android"
                  ]
                else
                  [ ]
              )
              ++
              # iOS targets
              (
                if roles.ios then
                  [
                    "aarch64-apple-ios"
                    "aarch64-apple-ios-sim"
                  ]
                else
                  [ ]
              );
            # TODO: find a way to include "aarch64-pc-windows-msvc" "x86_64-pc-windows-msvc"
          })
          clippy
          rust-analyzer
          cargo
          just
          gcc
          openssl
          pkg-config
        ];

        slintTools = with pkgs; [
          slint-lsp
          slint-viewer
        ];

        extraTools = with pkgs; [
          codebook # efficient spellchecker for code
          nil # Nix language server (older)
          nixd # Nix language server (newer)
          nixfmt # Official Nix formatter
          nushell # powerful and pragmatic shell written in Rust
          tombi # TOML language server
          #jetbrains.rust-rover
        ];

        linuxLdLibraryPath =
          if roles.linux then
            [
              pkgs.libGL # REQUIRED to run the Slint app on Linux with OpenGL (Femtovg)
              pkgs.wayland # REQUIRED to build and run the app on Linux with Wayland
              pkgs.libxkbcommon # REQUIRED (this prevents `called `Result::unwrap()` on an `Err` value: XKBNotFound`)
              pkgs.fontconfig
              pkgs.fontconfig.dev
            ]
          else
            [ ];
        linuxTools = if roles.linux then [
          pkgs.fontconfig
          pkgs.fontconfig.dev
        ] else [ ];

        macosLdLibraryPath = if roles.macos then [ ] else [ ];
        macosTools = if roles.macos then [ ] else [ ];

        iosLdLibraryPath = if roles.ios then [ ] else [ ];
        iosTools =
          if roles.ios then
            [
              pkgs.xcodegen
              pkgs.gettext # provides envsubst for template substitution; only used in the shellHook
            ]
          else
            [ ];

        androidLdLibraryPath =
          if roles.android then
            if isLinux then
              [
                pkgs.vulkan-loader
                pkgs.libGL # REQUIRED to run the emulator with OpenGL (common fallback from Vulkan to OpenGL in case of `useVulkanComposition: false`)
              ]
            else
              [ ] # macOS does not need vulkan-loader and libGL
          else
            [ ];

        androidTools =
          if roles.android then
            [
              pkgs.gradle # REQUIRED to build and run the app
              pkgs.jdk17 # REQUIRED to build and run the app
              androidSdk # REQUIRED to build and run the app
              pkgs.cargo-apk # REQUIRED to build and run the app with Cargo
              pkgs.cargo-ndk # REQUIRED to build and run the app with Rust
            ]
          else
            [ ];

        # complements androidTools
        androidSdk =
          if roles.android then
            let
              androidEnv = pkgs.androidenv.override { licenseAccepted = true; };
              composition = androidEnv.composeAndroidPackages {
                includeNDK = true;
                cmdLineToolsVersion = "8.0";
                platformToolsVersion = "35.0.2";
                buildToolsVersions = [
                  #"30.0.3"
                  #"33.0.2"
                  #"34.0.0"
                  "35.0.1"
                  # "36.0.0"
                ];
                platformVersions = [
                  #"30" # Android 11
                  #"31" # Android 12
                  #"32" # Android 12L
                  #"33" # Android 13
                  #"34" # Android 14
                  "35" # Android 15 (Note: v. 15 is necessary to compile ATM)
                  # "36" # Android 16 # apparently not supported yet by
                ];
                abiVersions = abiVersions;
                includeEmulator = true;
                includeSystemImages = true;
                systemImageTypes = [ "google_apis_playstore" ];
              };
            in
              composition.androidsdk
          else
            null;

        # complements androidSdk
        abiVersions =
          if system == "x86_64-linux" || system == "x86_64-darwin" then
            [ "x86_64" ]
          else if system == "aarch64-linux" || system == "aarch64-darwin" then
            [ "arm64-v8a" ]
          else
            builtins.throw "Unsupported architecture: ${system}. Only x86_64-linux, x86_64-darwin, aarch64-linux, and aarch64-darwin are supported for Android development.";
      in
        {
        packages = {

          carris-desktop = pkgs.rustPlatform.buildRustPackage {
            pname = "carris-desktop";
            version = "0.1.0";
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            src = ./.;

            nativeBuildInputs = with pkgs; [
              pkg-config
              rustPlatform.bindgenHook
            ];


            buildInputs =
              rustTools
              ++ slintTools
              ++ extraTools
              ++ (if roles.linux then linuxTools else [ ])
              ++ (if roles.macos then macosTools else [ ])
              ++ (if roles.android then androidTools else [ ])
              ++ (if roles.ios then iosTools else [ ]);

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
              (if roles.linux then linuxLdLibraryPath else [ ])
              ++ (if roles.macos then macosLdLibraryPath else [ ])
              ++ (if roles.android then androidLdLibraryPath else [ ])
              ++ (if roles.ios then iosLdLibraryPath else [ ])
            );

          };

          default = self.packages.${system}.carris-desktop;
          slint-preview = pkgs.writeShellApplication {
            name = "slint-preview";
            runtimeInputs = with pkgs; [ slint-viewer ];
            text = ''
            exec slint-viewer ui/main.slint \
            -L 'material=./material-1.0/material.slint' \
            "$@"
            '';
          };
        };

        apps.slint-preview = flake-utils.lib.mkApp {
          drv = self.packages.${system}.slint-preview;
        };

        devShells.default = pkgs.mkShell {
          name = "dev";

          buildInputs =
            rustTools
            ++ slintTools
            ++ extraTools
            ++ (if roles.linux then linuxTools else [ ])
            ++ (if roles.macos then macosTools else [ ])
            ++ (if roles.android then androidTools else [ ])
            ++ (if roles.ios then iosTools else [ ]);

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            (if roles.linux then linuxLdLibraryPath else [ ])
            ++ (if roles.macos then macosLdLibraryPath else [ ])
            ++ (if roles.android then androidLdLibraryPath else [ ])
            ++ (if roles.ios then iosLdLibraryPath else [ ])
          );

          # Android graphics envars
          QT_QPA_PLATFORM = if (roles.android && isLinux) then "wayland;xcb" else null;
          LIBGL_DRIVERS_PATH = if roles.android then "/run/opengl-driver/lib/dri" else null;
          VK_ICD_FILENAMES =
            if roles.android then
              if isLinux then
                "/run/opengl-driver/share/vulkan/icd.d/nvidia_icd.json:/run/opengl-driver/share/vulkan/icd.d/intel_icd.x86_64.json:/run/opengl-driver/share/vulkan/icd.d/radeon_icd.x86_64.json:/run/opengl-driver/share/vulkan/icd.d/gfxstream_vk_icd.x86_64.json"
              else
                null # in case of macOS (or another platform in the future), Vulkan is not used
            else
              null; # in case Android support is disabled, the envar is not used.

          # Android envars
          ANDROID_HOME = if roles.android then "${androidSdk}/libexec/android-sdk" else null;
          ANDROID_NDK_ROOT = if roles.android then "${androidSdk}/libexec/android-sdk/ndk-bundle" else null;
          JAVA_HOME = if roles.android then pkgs.jdk17.home else null;
          GRADLE_OPTS =
            if roles.android then
              "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidSdk}/libexec/android-sdk/build-tools/35.0.0/aapt2" # update as required
            else
              null;

          # desktop envars
          # SLINT_BACKEND = "winit-femtovg"; # not necessary to set. Uncomment/change envar if needed.
          # SLINT_LIVE_PREVIEW: ... do not set this envar here. Just use the envar before calling `cargo run` (desktop only). Using the envar with mobile will lead to issues.

          # iOS app compiler (clang)
          # By default, Nix's cc-wrapper/clang is used. However, it is not designed with multi-target compilers in mind.
          # It only works with macOS's SDK. Therefore, we need to use the system's clang for iOS builds.
          CARGO_TARGET_AARCH64_APPLE_IOS_LINKER = if roles.ios then "/usr/bin/clang" else null;
          CARGO_TARGET_AARCH64_APPLE_IOS_SIM_LINKER = if roles.ios then "/usr/bin/clang" else null;

          # In case a similar linker is needed for macOS builds, uncomment the following line(s):
          # CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER = if roles.ios then "/usr/bin/clang" else null;
          # CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER = if roles.ios then "/usr/bin/clang" else null;

          shellHook = ''
            echo "Loaded roles:"
            echo "  android: ${toString roles.android}"
            echo "  ios:     ${toString roles.ios}"
            echo "  linux:   ${toString roles.linux}"
            echo "  macos:   ${toString roles.macos}"

            ${pkgs.lib.optionalString roles.ios ''
              # unset the following 2 envars (by default, they contain paths in the nix store)
              # export DEVELOPER_DIR="/Applications/Xcode.app/Contents/Developer" # explicit alternative to the line below
              unset DEVELOPER_DIR # implicit
              # export SDKROOT="/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.1.sdk" # explicit alternative to the line below
              unset SDKROOT # implicit
              export PATH=$(echo "$PATH" | tr ':' '\n' | grep -v xcbuild | paste -sd:) # this basically removes xcbuild's nix-store-path entry from the PATH

              # Create iOS build script from template with cargo path replacement
              # Save current directory and navigate to project root (this way nix develop can be run also from slint-app, or any other folder/subfolder)
              pushd . > /dev/null
              # Find project root by looking for flake.nix
              while [[ ! -f "flake.nix" && "$PWD" != "/" ]]; do
                cd ..
              done
              if [[ ! -f "flake.nix" ]]; then
                echo "Error: Could not find project root (flake.nix not found)"
                popd > /dev/null
                return 1
              fi
              # Set environment variable and use envsubst for template substitution
              export NIX_STORE_CARGO_PATH_BIN="${pkgs.cargo}/bin" # Note: this might not always get set to the correct path for some weird reason... Manually override the path in the file build_for_ios_with_cargo.bash in that case.
              envsubst '$NIX_STORE_CARGO_PATH_BIN' < slint-app/build_for_ios_with_cargo.bash.template > slint-app/build_for_ios_with_cargo.bash
              chmod +x slint-app/build_for_ios_with_cargo.bash
              unset NIX_STORE_CARGO_PATH_BIN
              # Restore original directory
              popd > /dev/null
            ''}
          '';
        };
      }
    );
}
