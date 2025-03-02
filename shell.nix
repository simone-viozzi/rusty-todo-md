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

  # Define the CodeLLDB extension directory and its lldb/lib subdirectory
  codelldbExt = "$HOME/.vscode/extensions/vadimcn.vscode-lldb-1.11.4";
  codelldbLib = "${codelldbExt}/lldb/lib";
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
    pre-commit
    patchelf
    lldb
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

    # Add CodeLLDB's lldb/lib directory to LD_LIBRARY_PATH so libpython312.so is found.
    if [ -d "${codelldbLib}" ]; then
      export LD_LIBRARY_PATH="${codelldbLib}:$LD_LIBRARY_PATH"
    fi

    unset SOURCE_DATE_EPOCH
    pip install --upgrade wheel setuptools
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

    # --- Automatically patch codelldb adapter if found ---
    for file in ~/.vscode/extensions/vadimcn.vscode-lldb-*/adapter/codelldb; do
      if [ -x "$file" ]; then
        echo "Patching codelldb adapter: $file"
        # Set the interpreter to the proper glibc dynamic linker.
        patchelf --set-interpreter "${pkgs.glibc}/lib/ld-linux-x86-64.so.2" "$file"
        # Include both the zlib lib and the CodeLLDB lldb/lib in the rpath.
        patchelf --set-rpath "${pkgs.zlib}/lib:${codelldbLib}" "$file"
      fi
    done

    # ---------------------------------------------------------
  '';
}
