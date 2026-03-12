#!/bin/bash
# KoadOS Path Management Utility

CONFIG_FILE="$HOME/.koad-os/config/filesystem.toml"

list_mappings() {
    python3 -c "import tomllib, os; config = tomllib.load(open('$CONFIG_FILE', 'rb')); print('\n'.join([f'{k}: {v}' for k, v in config.get('filesystem', {}).get('mappings', {}).items()]))"
}

get_mapping() {
    python3 -c "import tomllib, os; config = tomllib.load(open('$CONFIG_FILE', 'rb')); print(config.get('filesystem', {}).get('mappings', {}).get('$1', 'Not Found'))"
}

if [ -z "$1" ]; then
    echo "--- KoadOS Path Mappings ---"
    list_mappings
else
    get_mapping "$1"
fi
