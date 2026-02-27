#!/bin/bash
# KoadOS WSL Bridge - Translates Windows paths to WSL formats for koad CLI

translated_args=()
for arg in "$@"; do
    # Check if the argument looks like a Windows path (drive letter + colon + backslash or starting with double backslash)
    if [[ "$arg" == [a-zA-Z]:\\* ]] || [[ "$arg" == \\\\* ]]; then
        # Use wslpath -u to convert Windows path to WSL format
        translated=$(wslpath -u "$arg" 2>/dev/null || echo "$arg")
        translated_args+=("$translated")
    else
        translated_args+=("$arg")
    fi
done

# Execute koad with translated arguments
exec koad "${translated_args[@]}"
