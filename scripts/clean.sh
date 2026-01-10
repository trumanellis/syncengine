#!/bin/bash
# Quick cleanup script for Synchronicity Engine
# Usage: ./scripts/clean.sh [db|build|all]
#
# Commands:
#   db      Reset database (backup + delete)
#   build   Clean build artifacts
#   all     Reset everything (db + build)
#   (none)  Kill stuck processes only

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Determine data directory
if [ -d "$HOME/Library/Application Support/syncengine" ]; then
    DATA_DIR="$HOME/Library/Application Support/syncengine"
elif [ -d "$HOME/.local/share/syncengine" ]; then
    DATA_DIR="$HOME/.local/share/syncengine"
else
    DATA_DIR=""
fi

DB_FILE="$DATA_DIR/syncengine.redb"

kill_processes() {
    echo -e "${CYAN}Killing syncengine processes...${NC}"
    pkill -f "stress_tests" 2>/dev/null || true
    pkill -f "syncengine-desktop" 2>/dev/null || true
    pkill -f "syncengine-cli" 2>/dev/null || true
    pkill -f "syncengine_desktop" 2>/dev/null || true
    sleep 1
    echo -e "${GREEN}Done.${NC}"
}

reset_db() {
    if [ -z "$DATA_DIR" ]; then
        echo -e "${YELLOW}Data directory not found. Skipping database reset.${NC}"
        return
    fi

    if [ -f "$DB_FILE" ]; then
        echo -e "${CYAN}Backing up and removing database...${NC}"
        BACKUP_FILE="${DB_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
        cp "$DB_FILE" "$BACKUP_FILE"
        rm "$DB_FILE"
        echo -e "${GREEN}Database backed up to: ${BACKUP_FILE}${NC}"
        echo -e "${GREEN}Fresh database will be created on next launch.${NC}"
    else
        echo -e "${YELLOW}No database file found. Nothing to reset.${NC}"
    fi
}

clean_build() {
    echo -e "${CYAN}Cleaning build artifacts...${NC}"
    cargo clean 2>/dev/null || true
    echo -e "${GREEN}Build artifacts cleaned.${NC}"
}

show_usage() {
    echo "Synchronicity Engine Cleanup Script"
    echo ""
    echo "Usage: ./scripts/clean.sh [command]"
    echo ""
    echo "Commands:"
    echo "  (none)    Kill stuck processes only"
    echo "  db        Reset database (backup + delete)"
    echo "  build     Clean build artifacts (cargo clean)"
    echo "  all       Reset everything (processes + db + build)"
    echo ""
    echo "Examples:"
    echo "  ./scripts/clean.sh          # Just kill stuck processes"
    echo "  ./scripts/clean.sh db       # Reset database"
    echo "  ./scripts/clean.sh all      # Full reset"
}

case "${1:-}" in
    db)
        kill_processes
        reset_db
        ;;
    build)
        clean_build
        ;;
    all)
        kill_processes
        reset_db
        clean_build
        ;;
    -h|--help|help)
        show_usage
        ;;
    "")
        kill_processes
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        echo ""
        show_usage
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}=== Cleanup complete ===${NC}"
echo ""
echo "To launch the app:"
echo "  cargo run --bin syncengine-desktop"
