#!/bin/sh

set -e

if [ $# -ne 1 ]; then
    printf "Usage: ./update_disk.sh [userapp path]\n"
    exit 1
fi

FILE=$1
DISK=./disk.img
MNT=./mnt

if [ ! -f "$FILE" ]; then
    echo "❌ File '$FILE' doesn't exist!"
    exit 1
fi

if [ ! -f "$DISK" ]; then
    echo "❌ '$DISK' doesn't exist! Please 'make disk_img' first."
    exit 1
fi

echo "✅ Writing '$FILE' into disk image '$DISK'..."

mkdir -p "$MNT"

# macOS
if [ "$(uname)" = "Darwin" ]; then
    echo "🟦 Detected macOS, using hdiutil..."
    DEVICE=$(hdiutil attach "$DISK" -mountpoint "$MNT" -nobrowse -readwrite | grep "$MNT" | awk '{print $1}')
    sudo mkdir -p "$MNT/sbin"
    sudo cp "$FILE" "$MNT/sbin/origin"
    sync
    hdiutil detach "$DEVICE"
    echo "✅ Done (macOS)"
else
    echo "🟩 Detected Linux, using mount..."
    mount -o loop "$DISK" "$MNT"
    mkdir -p "$MNT/sbin"
    cp "$FILE" "$MNT/sbin/origin"
    sync
    umount "$MNT"
    echo "✅ Done (Linux)"
fi

# 清理挂载点（避免 macOS root 创建残留）
echo "🧹 Cleaning up mount point..."
sudo rm -rf "$MNT"/* "$MNT"/.* 2>/dev/null || true
rmdir "$MNT" 2>/dev/null || true