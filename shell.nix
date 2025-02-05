{
  pkgs ? import <nixpkgs> { },
}:

let
  pythonPackages = pkgs.python311Packages; # Change to Python 3.10
in pkgs.mkShell rec {
  name = "pre-commit-todo";
  venvDir = "./.venv";

  buildInputs = with pkgs; [
    rustc # Rust compiler
    cargo # Rust package manager
    rustfmt # Rust code formatter
    clippy # Rust linter
    gcc # Required for crates needing C compilers
    pkg-config # Helps locate libraries like OpenSSL
    openssl # OpenSSL library for crates like openssl-sys
    git
    git-crypt
    stdenv.cc.cc.lib
    stdenv.cc.cc # jupyter lab needs
    pythonPackages.python
    pythonPackages.pyzmq # Adding pyzmq explicitly
    pythonPackages.venvShellHook
    pythonPackages.pip
    pythonPackages.ruff
    pre-commit
  ];

  postVenvCreation = ''
    unset SOURCE_DATE_EPOCH
  '';

  # Set the source path for Rust tooling (e.g., rust-analyzer)
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  pre-commit = pkgs.pre-commit;

  postShellHook = ''
    # allow pip to install wheels
    unset SOURCE_DATE_EPOCH

    pip install --upgrade wheel setuptools  

    echo "Environment setup complete."
  '';

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

    export RUST_BACKTRACE=1
    export CARGO_HOME=$HOME/.cargo
    export PATH=$CARGO_HOME/bin:$PATH
  '';
}
