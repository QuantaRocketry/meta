#!/bin/bash
set -e

# Cargo often passes '--' as the first argument. 
# We want the actual file path.
if [ "$1" = "--" ]; then
    ELF_FILE="$2"
else
    ELF_FILE="$1"
fi

# Ensure we have a file to work with
if [ -z "$ELF_FILE" ]; then
    echo "Error: No ELF file provided"
    exit 1
fi

BIN_FILE="${ELF_FILE}.bin"
CONV_SCRIPT="./lib/uf2/utils/uf2conv.py"

echo "Converting $ELF_FILE to binary..."
if command -v llvm-objcopy >/dev/null 2>&1; then
    OBJCOPY="llvm-objcopy"
elif command -v arm-none-eabi-objcopy >/dev/null 2>&1; then
    OBJCOPY="arm-none-eabi-objcopy"
elif command -v rust-objcopy >/dev/null 2>&1; then
    OBJCOPY="rust-objcopy"
else
    echo "Error: No suitable objcopy found. Please install llvm-objcopy, arm-none-eabi-objcopy, or rust-objcopy."
    exit 1
fi

$OBJCOPY -O binary "$ELF_FILE" "$BIN_FILE"

echo "Converting to UF2..."
python3 "$CONV_SCRIPT" -c -b 0x27000 -f 0xada52840 -o "$ELF_FILE.uf2" "$BIN_FILE"

echo "Done! Produced flash.uf2"