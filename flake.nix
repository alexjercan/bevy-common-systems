{
  description = "A basic flake for my Bevy Game";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        # Optional: use external flake logic, e.g.
        # inputs.foo.flakeModules.default
      ];
      flake = {
        # Put your original flake attributes here.
      };
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      perSystem = {
        system,
        ...
      }: let
        overlays = [ (import inputs.rust-overlay) ];
        pkgs = import inputs.nixpkgs {
          inherit system overlays;
        };
        rustNightly = pkgs.rust-bin.nightly.latest.default.override {
          extensions = ["rust-src" "clippy" "rustfmt"];
          targets = ["wasm32-unknown-unknown"];
        };
      in {
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            openssl
            trunk
            wasm-pack
            rustNightly
            clippy
            rust-analyzer
            pkg-config
            llvmPackages.bintools
            nodejs # for the web/ showcase (webpack + TypeScript)
          ];

          buildInputs = with pkgs; [
            udev
            alsa-lib-with-plugins
            vulkan-loader
            libx11
            libxcursor
            libxi
            libxrandr # To use the x11 feature
            libxkbcommon
            wayland # To use the wayland feature
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          RUST_BACKTRACE = 1;

          RUST_SRC_PATH = "${rustNightly}/lib/rustlib/src/rust/library";

          # Keep `cargo test` inside RAM. It links one full Bevy+avian binary
          # per target (lib unittest, each of the 15 examples, each of the 60
          # doctests) and rust-lld holds a multi-GB image per link, so the peak
          # is roughly (links in flight) x (binary size). Cargo defaults the
          # first factor to the core count, which on a many-core desktop far
          # outruns RAM. Cargo.toml's [profile.dev] caps the second factor;
          # this caps the first.
          #
          # NOTE: derived, not a constant. Two very different machines enter
          # this shell -- a 24-core/31 GB desktop and a 4-core/16 GB GitHub
          # runner (pages.yml builds via `nix develop`). A hardcoded cap would
          # be far too high for one or a needless slowdown for the other, and
          # committing one to .cargo/config.toml would also hit ci.yml, which
          # runs bare cargo on the small runner and needs no cap at all.
          # ~4 GB of headroom per concurrent link matches the measured peak in
          # tasks/20260731-210044/NOTES.md. Re-measure with
          # ./scripts/sample-peak-rss.sh after adding examples or doctests.
          #
          # CARGO_BUILD_JOBS covers the lib and example links. RUST_TEST_THREADS
          # covers the doctest links, which cargo does NOT pass a job limit to:
          # rustdoc's harness would otherwise link one doctest per core. Both
          # are exported only if the user has not already set them, and the
          # whole block is skipped where /proc/meminfo does not exist, so the
          # darwin systems this flake also declares are left alone.
          shellHook = ''
            _bcs_cap() {
              local cores mem_gb cap
              cores=$(nproc)
              mem_gb=$(awk '/MemTotal/ {printf "%d", $2 / 1048576}' /proc/meminfo)
              cap=$(( mem_gb / 4 ))
              [ "$cap" -lt 1 ] && cap=1
              [ "$cap" -gt "$cores" ] && cap=$cores
              echo "$cap"
            }
            if [ -r /proc/meminfo ]; then
              export CARGO_BUILD_JOBS="''${CARGO_BUILD_JOBS:-$(_bcs_cap)}"
              export RUST_TEST_THREADS="''${RUST_TEST_THREADS:-$(_bcs_cap)}"
            fi
            unset -f _bcs_cap
          '';
        };
      };
    };
}
