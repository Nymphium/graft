#!/bin/bash
set -e

# Generate the supported languages list
cat <<'EOL' > SUPPORTED_LANGUAGES.md
Supported Languages
===

The following languages are currently supported by Graft:
EOL

# Run graft to list languages and append to the file
cargo run -- --list-languages >> SUPPORTED_LANGUAGES.md

echo "Generated SUPPORTED_LANGUAGES.md"
