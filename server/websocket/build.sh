mkdir output
cp script/* output 2>/dev/null
cp -r src/conf.yaml output/ 2>/dev/null
cargo build --bin main --release
cp target/release/main output
chmod +x output/*