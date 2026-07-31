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
          # The divisor is 6, i.e. ~6 GB of RAM budgeted per concurrent link.
          # That is roughly 2x the measured cost: the largest single rust-lld
          # was 3.0 GB and the whole-run peak divided by the cap comes to
          # ~2.7 GB per job, the rest being rustc and the driver. The margin is
          # deliberate -- the peak is what matters, and overshooting it means
          # swap, not a slow build. See tasks/20260731-210044/NOTES.md for the
          # four measured configurations; re-measure with
          # ./scripts/sample-peak-rss.sh after adding examples or doctests.
          #
          # NOTE: 6, not 4. A divisor of 4 gives 7 here and was measured at
          # 18.4 GB on `cargo test --features debug` -- over this repo's 16 GB
          # target, because `--features debug` links egui into every binary and
          # is the heaviest configuration, not the default one the older figure
          # came from. At 6 the same command peaks at 13.5 GB.
          #
          # Concrete values: 5 on the desktop (min(24, 31/6)). On a 4-core/16 GB
          # runner it is 2 -- MemTotal there reports ~15.6 GiB and the awk
          # truncates to 15, so pages.yml builds at 2 jobs rather than its
          # default 4. That halving is accepted rather than tuned away: 4
          # concurrent links at ~3 GB each is ~12 GB of a 15.6 GiB runner before
          # rustc overhead, which is the OOM this whole file exists to prevent.
          # The wasm example builds are not the CI bottleneck.
          #
          # CARGO_BUILD_JOBS covers the lib and example links. RUST_TEST_THREADS
          # covers the doctest links, which cargo does NOT pass a job limit to:
          # rustdoc's harness would otherwise link one doctest per core. Both
          # are exported only if the user has not already set them, and the
          # whole block is skipped where /proc/meminfo does not exist, so the
          # darwin systems this flake also declares are left alone.
          #
          # Two costs, both known and accepted. RUST_TEST_THREADS is overloaded:
          # it caps doctest LINKING, but being a shell-wide export it also caps
          # test EXECUTION for every libtest harness here -- cheap, because
          # these tests are pure math. CARGO_BUILD_JOBS is likewise not scoped
          # to linking, so a cold ~400-crate dependency compile also runs at the
          # cap, where rustc rather than rust-lld is the memory profile and the
          # cap buys nothing. Cargo has no separate link-jobs knob, so both are
          # forced. Override either per command when you want the cores back:
          # `CARGO_BUILD_JOBS=24 cargo build`.
          shellHook = ''
            _bcs_cap() {
              local cores mem_gb cap
              cores=$(nproc)
              mem_gb=$(awk '/MemTotal/ {printf "%d", $2 / 1048576}' /proc/meminfo)
              cap=$(( mem_gb / 6 ))
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
