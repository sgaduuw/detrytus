#!/bin/sh
# Rebuild the LLVM IR the ll tests translate (rust/tc/tests/ll/*.ll)
# from the Rust programs beside it, as docs/rust.md builds a program:
# for riscv32imac-unknown-none-elf, with core built from source and
# panics that abort at once. Needs a Rust toolchain with the target's
# rust-std and rust-src:
#   rustup target add riscv32imac-unknown-none-elf
#   rustup component add rust-src
set -eu
here=$(cd "$(dirname "$0")" && pwd)
work=${TMPDIR:-/tmp}/detrytus-ll-tests
mkdir -p "$work/src"
cat > "$work/Cargo.toml" <<'TOML'
[package]
name = "prog"
version = "0.1.0"
edition = "2024"
[lib]
crate-type = ["staticlib"]
[profile.release]
opt-level = "z"
panic = "abort"
codegen-units = 1
TOML
for src in "$here"/*.rs; do
    name=$(basename "$src" .rs)
    cp "$src" "$work/src/lib.rs"
    rm -f "$work"/target/riscv32imac-unknown-none-elf/release/deps/prog-*.ll
    (cd "$work" && RUSTC_BOOTSTRAP=1 \
        RUSTFLAGS="--emit=llvm-ir,link -Zunstable-options -Cpanic=immediate-abort" \
        cargo build --quiet --release -Zbuild-std=core --target riscv32imac-unknown-none-elf)
    # What main reaches of the program and core, as one module.
    deps="$work"/target/riscv32imac-unknown-none-elf/release/deps
    cargo run --quiet --manifest-path "$here/../../../../Cargo.toml" -p detrytus-cli -- \
        ll --prune "$deps"/prog-*.ll "$deps"/core-*.ll -o "$here/$name.ll"
    sed -i.bak "s|$deps/||g" "$here/$name.ll" && rm "$here/$name.ll.bak"
    echo "$name.ll"
done
