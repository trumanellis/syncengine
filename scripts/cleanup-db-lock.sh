#!/bin/bash
# Cleanup script for Synchronicity Engine database lock issues
# Usage: ./scripts/cleanup-db-lock.sh [--reset]
#
# Options:
#   --reset    Back up and delete the database (fresh start)

set -e

RESET_DB=false
if [ "$1" = "--reset" ]; then
    RESET_DB=true
fi

echo "=== Synchronicity Engine Database Lock Cleanup ==="
echo ""

# Find and kill any syncengine-related processes (except IDE extensions)
echo "Checking for running syncengine processes..."
PROCS=$(pgrep -fl "syncengine" 2>/dev/null | grep -v "Antigravity\|language_server" || true)

if [ -n "$PROCS" ]; then
    echo "Found running processes:"
    echo "$PROCS"
    echo ""
    echo "Killing processes..."
    pkill -f "stress_tests" 2>/dev/null || true
    pkill -f "syncengine-desktop" 2>/dev/null || true
    pkill -f "syncengine-cli" 2>/dev/null || true
    sleep 1
    echo "Done."
else
    echo "No syncengine processes found."
fi

echo ""

# Check data directory location (macOS vs Linux)
if [ -d "$HOME/Library/Application Support/syncengine" ]; then
    DATA_DIR="$HOME/Library/Application Support/syncengine"
elif [ -d "$HOME/.local/share/syncengine" ]; then
    DATA_DIR="$HOME/.local/share/syncengine"
else
    echo "Data directory not found. No cleanup needed."
    exit 0
fi

echo "Data directory: $DATA_DIR"
DB_FILE="$DATA_DIR/syncengine.redb"

# Check if database file is locked
if [ -f "$DB_FILE" ]; then
    LOCKERS=$(lsof "$DB_FILE" 2>/dev/null || true)
    if [ -n "$LOCKERS" ]; then
        echo ""
        echo "WARNING: Database file is still locked by:"
        echo "$LOCKERS"
        echo ""
        read -p "Force kill these processes? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            lsof -t "$DB_FILE" 2>/dev/null | xargs kill -9 2>/dev/null || true
            echo "Killed."
        fi
    else
        echo "Database file is not locked by any process."
    fi

    # Reset database if requested
    if [ "$RESET_DB" = true ]; then
        echo ""
        echo "Resetting database..."
        BACKUP_FILE="${DB_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
        cp "$DB_FILE" "$BACKUP_FILE"
        rm "$DB_FILE"
        echo "Database backed up to: $BACKUP_FILE"
        echo "Original database removed. App will create a fresh database."
    fi
else
    echo "Database file not found at $DB_FILE"
fi

echo ""
echo "=== Cleanup complete ==="
echo ""
echo "You can now start the application:"
echo "  cargo run --bin syncengine-desktop"
echo ""
echo "If you still have issues, try:"
echo "  ./scripts/cleanup-db-lock.sh --reset"
