#!/bin/bash
# KoadOS Path Management Utility

CONFIG_FILE="$HOME/.koad-os/koad.json"

list_mappings() {
    python3 -c "import json, os; config = json.load(open('$CONFIG_FILE')); print('\n'.join([f'{k}: {v}' for k, v in config.get('filesystem', {}).get('mappings', {}).items()]))"
}

get_mapping() {
    python3 -c "import json, os; config = json.load(open('$CONFIG_FILE')); print(config.get('filesystem', {}).get('mappings', {}).get('$1', 'Not Found'))"
}

if [ -z "$1" ]; then
    echo "--- KoadOS Path Mappings ---"
    list_mappings
else
    get_mapping "$1"
fi
