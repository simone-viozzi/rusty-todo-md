{
  pkgs ? import <nixpkgs> { },
  overlays ? [ ],
}:

let
  rust-overlay = import (
    builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"
  );
  pkgs = import <nixpkgs> { overlays = [ rust-overlay ]; };

  rust = pkgs.rust-bin.stable.latest.default.override {
    extensions = [
      "rust-src"
      "cargo"
      "rustc"
      "clippy"
      "rustfmt"
    ];
  };

  pythonPackages = pkgs.python312Packages;
in

pkgs.mkShell {
  name = "comment-parser";
  venvDir = "./.venv";

  # Define dynamic linker variables.
  NIX_LD = pkgs.lib.fileContents "${pkgs.stdenv.cc}/nix-support/dynamic-linker";
  NIX_LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.stdenv.cc.cc
    pkgs.openssl
    pkgs.zlib
    pkgs.lldb
    #pkgs.lldb.out
  ];

  buildInputs = with pkgs; [
    rust
    rust-analyzer
    pkg-config
    openssl
    git
    pre-commit
    lldb
    llvmPackages.libllvm
    gcc
    zlib
    zlib.out
    pythonPackages.python
    pythonPackages.pyzmq
    pythonPackages.venvShellHook
    pythonPackages.pip
    pythonPackages.ruff
    pythonPackages.click
    pythonPackages.pathspec
    pythonPackages.tqdm
    pythonPackages.pytest
    pythonPackages.wheel
    patchelf
    maturin
  ];

  postVenvCreation = ''
    unset SOURCE_DATE_EPOCH
  '';

  pre-commit = pkgs.pre-commit;

  postShellHook = ''
    export RUST_BACKTRACE=1
    export CARGO_HOME=$HOME/.cargo
    export PATH=$CARGO_HOME/bin:$PATH
    export RUST_SRC_PATH="${rust}/lib/rustlib/src/rust/library"

    # Ensure our dynamic linker settings remain active.
    export NIX_LD
    export NIX_LD_LIBRARY_PATH
    export LD_LIBRARY_PATH=${pkgs.zlib}/lib:$LD_LIBRARY_PATH

    pip install --upgrade wheel setuptools
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

    export LLDB_DEBUGSERVER_PATH="${pkgs.lldb.out}/bin/lldb-server"

    # Create a local directory for LLDB symlinks
    LLDB_BIN_DIR="./lldb-bin"
    mkdir -p "$LLDB_BIN_DIR"

    # Symlink liblldb.so from the lldb.lib output to the local directory
    ln -sf "${pkgs.lldb}/lib/liblldb.so" "$LLDB_BIN_DIR/liblldb.so"

    # Symlink lldb-server from the lldb.out output to the local directory
    ln -sf "${pkgs.lldb.out}/bin/lldb-server" "$LLDB_BIN_DIR/lldb-server"

    echo "Created local LLDB bin directory at $(pwd)/lldb-bin"
    echo "Set VSCode 'lldb.library' to $(pwd)/lldb-bin/liblldb.so"

    # Patch the codelldb adapter executable with the correct dynamic linker.
    if [ -f "$HOME/.vscode/extensions/vadimcn.vscode-lldb-1.11.4/adapter/codelldb" ]; then
      echo "Patching codelldb adapter..."
      patchelf --set-interpreter "$NIX_LD" "$HOME/.vscode/extensions/vadimcn.vscode-lldb-1.11.4/adapter/codelldb"
    else
      echo "codelldb adapter not found, skipping patch."
    fi

  '';
}
