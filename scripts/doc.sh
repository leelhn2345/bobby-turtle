if ! [ -x "$(command -v cargo makedocs)" ]; then
    echo "Error: cargo makedocs is not installed." >&2
    echo "Use:" >&2
    echo "      cargo install cargo-makedocs" >&2
    exit 1
fi

cargo makedocs --open \
    -i dptree # include sub-dependencies
