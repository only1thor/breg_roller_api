# Bedrift roller
prosjekt for å finne personers rolle i en bedrift. 

## Nix shell
[kilde](https://gutier.io/post/development-using-rust-with-nix/) for oppsett av shell.nix

fra kilden:

# Setting up a Rust environment in Nix

`
Julio César
@ZzAntares
`

When using Nix, we have different mechanisms at our disposal to setup a development environment for Rust, in this post we’ll explore the most common alternatives out there and the nicest of them all.
## Rust provided by Nixpkgs

The most straight forward way of getting a Rust installation up to speed is by using a shell.nix file with the following expression:
```
{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
  ];

  RUST_BACKTRACE = 1;
}
```
With that file in our project folder it is enough to call nix-shell to load an environment with Rust and most essential packages provided to us, this is nice, however if you reference a stable channel the versions of these packages you get could be quite old, of course one can choose to use unstable or just pin a newer commit:
```
{ nixpkgs ? import <nixpkgs> { }}:

let
  pinnedPkgs = nixpkgs.fetchFromGitHub {
    owner  = "NixOS";
    repo   = "nixpkgs";
    rev    = "1fe6ed37fd9beb92afe90671c0c2a662a03463dd";
    sha256 = "1daa0y3p17shn9gibr321vx8vija6bfsb5zd7h4pxdbbwjkfq8n2";
  };
  pkgs = import pinnedPkgs {};
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      clippy
      rustc
      cargo
      rustfmt
      rust-analyzer
    ];

    RUST_BACKTRACE = 1;
  }
```
Aside from getting a more recent version of everything (by choosing a recent revision), pinning has the added benefit of making a reference to a fixed package set that will not move under us, of course this is at the cost of being leaved behind as Nixpkgs evolve and introduces newer versions of these packages over time.
## Rust provided by a Oxalica’s Nix Overlay

Either of these approaches we already mentioned is fine if we’re getting started with Nix and just want to get a feeling of what’s like to work with nix-shell environments, however, what if at some point we wish to use a nightly version of Rust? or maybe just a specific version of the Rust toolchain? the answer there is quite simple, Mozilla works on a Nix overlay that gives us access to these, the Nixpkgs manual covers this and it basically consists of referencing Mozilla’s Nixpkgs clone, including the Rust overlay and get our dependencies from there.

One caveat with this approach is that we might end up compiling the rust toolchain in our computer rather than just download pre-built binaries, and that my friends… is no fun, it would be desirable then to download already pre-built binaries for us instead, this is where oxalica’s rust-overlay comes in, which I think is the best way to setup Rust in Nix:
```
{ nixpkgs ? import <nixpkgs> { }}:

let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pinnedPkgs = nixpkgs.fetchFromGitHub {
    owner  = "NixOS";
    repo   = "nixpkgs";
    rev    = "1fe6ed37fd9beb92afe90671c0c2a662a03463dd";
    sha256 = "1daa0y3p17shn9gibr321vx8vija6bfsb5zd7h4pxdbbwjkfq8n2";
  };
  pkgs = import pinnedPkgs {
    overlays = [ (import rustOverlay) ];
  };
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      rust-bin.stable.latest.default
      rust-analyzer
    ];

    RUST_BACKTRACE = 1;
  }
```
The answer is including Oxalica’s Rust overlay while importing the pinned version of the Nixpkgs, by doing this we get the best of both worlds, a Nix overlay that provides the Rust toolchain over a pinned version of the Nixpkgs.

Take note that cargo, clippy and rustfmt are provided by the Rust toolchain so we don’t need to get those individually they all come in when using rust-bin.stable.latest.default, this means “use the latest stable Rust toolchain” but we can also require a specific version, for example to require 1.48.0 refer to the toolchain like so:
```
rust-bin.stable."1.48.0".default
```
and to use a specific version of Rust nightly use this:
```
rust-bin.nightly."2020-12-31".default
```
or the absolute latest for those that like to stand in the edge 👀:

```
rust-bin.nightly.latest.default
```

All of this is particularly useful in NixOS where it seems rustup doesn’t work properly or at least for me it did not.
