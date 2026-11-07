# pcba: board folders and assembly drawings for KiCad boards

test:
    cargo nextest run --workspace

lint:
    cargo fmt --all --check
    cargo clippy -p pcba --all-targets --quiet -- -D warnings

install:
    cargo build --profile deploy --locked -p pcba_cli
    if command -v tpm >/dev/null; then tpm install target/deploy/pcba; else cargo install --locked --path pcba_cli; fi

# assembly drawing of B153 into its fab/ folder (example)
fab-b153:
    cargo run -q -p pcba_cli -- fab ../hw/b153_typec_serial/source/b153a/b153a_typec_serial.kicad_pcb
