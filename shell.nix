{ cargo-toml-lint
, clippy
, essential-debugger
, mkShell
, rust-analyzer
, rustfmt
}:
mkShell {
  inputsFrom = [
    essential-debugger
  ];
  buildInputs = [
    cargo-toml-lint
    clippy
    rust-analyzer
    rustfmt
  ];
}
