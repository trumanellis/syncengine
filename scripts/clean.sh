#!/bin/bash
# Quick cleanup script for Synchronicity Engine
# Usage: ./scripts/clean.sh [db|instances|build|all]
#
# Commands:
#   db         Reset main database (backup + delete)
#   instances  Delete all test instance data (instance-*, syncengine-*)
#   build      Clean build artifacts
#   all        Reset everything (db + instances + build)
#   (none)     Kill stuck processes only

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Determine base data directory
if [ -d "$HOME/Library/Application Support" ]; then
    BASE_DIR="$HOME/Library/Application Support"
elif [ -d "$HOME/.local/share" ]; then
    BASE_DIR="$HOME/.local/share"
else
    BASE_DIR=""
fi

DATA_DIR="$BASE_DIR/syncengine"
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
    if [ -z "$BASE_DIR" ]; then
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
        echo -e "${YELLOW}No database file found at $DB_FILE${NC}"
    fi
}

reset_instances() {
    if [ -z "$BASE_DIR" ]; then
        echo -e "${YELLOW}Data directory not found. Skipping instance reset.${NC}"
        return
    fi

    echo -e "${CYAN}Cleaning test instance directories...${NC}"

    # Count what we'll delete
    local count=0

    # New style: instance-*
    for dir in "$BASE_DIR"/instance-*; do
        if [ -d "$dir" ]; then
            echo -e "  ${YELLOW}Removing: $(basename "$dir")${NC}"
            rm -rf "$dir"
            ((count++)) || true
        fi
    done

    # Old style: syncengine-* (but not the main syncengine dir)
    for dir in "$BASE_DIR"/syncengine-*; do
        if [ -d "$dir" ]; then
            echo -e "  ${YELLOW}Removing: $(basename "$dir")${NC}"
            rm -rf "$dir"
            ((count++)) || true
        fi
    done

    if [ $count -eq 0 ]; then
        echo -e "${YELLOW}No instance directories found.${NC}"
    else
        echo -e "${GREEN}Removed $count instance directory(s).${NC}"
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
    echo "  (none)     Kill stuck processes only"
    echo "  db         Reset main database (backup + delete)"
    echo "  instances  Delete all test instance data (instance-*, syncengine-*)"
    echo "  build      Clean build artifacts (cargo clean)"
    echo "  all        Reset everything (processes + db + instances + build)"
    echo ""
    echo "Examples:"
    echo "  ./scripts/clean.sh              # Just kill stuck processes"
    echo "  ./scripts/clean.sh instances    # Clean test instances (love, joy, etc.)"
    echo "  ./scripts/clean.sh db           # Reset main database"
    echo "  ./scripts/clean.sh all          # Full reset"
}

case "${1:-}" in
    db)
        kill_processes
        reset_db
        ;;
    instances)
        kill_processes
        reset_instances
        ;;
    build)
        clean_build
        ;;
    all)
        kill_processes
        reset_db
        reset_instances
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
echo "  ./se love joy    # Two test instances"
