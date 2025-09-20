#!/bin/bash
# Document Reference Validation Script
# Validates all document references in the process system

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Only output when there are issues (silent success pattern)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

ERRORS=0
WARNINGS=0

# Get current timestamp for age calculations
CURRENT_TIME=$(date +%s)
ONE_WEEK_AGO=$((CURRENT_TIME - 604800))  # 7 days in seconds
ONE_MONTH_AGO=$((CURRENT_TIME - 2592000)) # 30 days in seconds

# Function to check if file exists and check age
check_file() {
    local file="$1"
    local context="$2"
    local critical="${3:-false}"  # Third param indicates if file should be frequently updated

    if [[ -f "$file" ]]; then
        local file_time
        file_time=$(stat -c %Y "$file" 2>/dev/null || stat -f %m "$file" 2>/dev/null)
        local age_indicator=""

        if [[ -n "$file_time" ]]; then
            if [[ "$critical" == "critical" ]]; then
                # Critical files should be updated within a week
                if [[ $file_time -lt $ONE_WEEK_AGO ]]; then
                    age_indicator=" ${YELLOW}(>1 week old - consider updating)${NC}"
                    ((WARNINGS++))
                fi
            else
                # Regular files get warning after a month
                if [[ $file_time -lt $ONE_MONTH_AGO ]]; then
                    age_indicator=" ${YELLOW}(>1 month old)${NC}"
                    ((WARNINGS++))
                fi
            fi
        fi

        if [[ -n "$age_indicator" ]]; then
            echo -e "${GREEN}✅${NC} $file ${BLUE}($context)${NC}$age_indicator"
        fi
        return 0
    else
        echo -e "${RED}❌${NC} $file ${BLUE}($context)${NC} - FILE NOT FOUND"
        ((ERRORS++))
        return 1
    fi
}

# Function to check if directory exists
check_dir() {
    local dir="$1"
    local context="$2"

    if [[ ! -d "$dir" ]]; then
        echo -e "${RED}❌${NC} $dir/ ${BLUE}($context)${NC} - DIRECTORY NOT FOUND"
        ((ERRORS++))
        return 1
    fi
    return 0
}

# Function to check file references within documents
check_references_in_file() {
    local file="$1"
    local context="$2"

    if [[ ! -f "$file" ]]; then
        echo -e "${RED}❌${NC} Cannot check references in $file - file not found"
        ((ERRORS++))
        return 1
    fi

    # Don't print per-file messages unless there are issues

    # Extract file references (basic pattern matching)
    # Look for patterns like: docs/*, .eggs/*, bin/*, etc.
    local found_issues=0

    while IFS= read -r line; do
        # Skip empty lines and comments
        [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue

        # Look for file references
        if [[ "$line" =~ docs/[^[:space:]]+\.(txt|md) ]] || \
           [[ "$line" =~ \.eggs/[^[:space:]]+\.(txt|md) ]] || \
           [[ "$line" =~ bin/[^[:space:]]+\.sh ]]; then

            # Extract the file path
            local ref_file
            ref_file=$(echo "$line" | grep -oE '(docs/[^[:space:]]+\.(txt|md)|\.eggs/[^[:space:]]+\.(txt|md)|bin/[^[:space:]]+\.sh)' | head -1)

            if [[ -n "$ref_file" ]]; then
                if [[ ! -f "$ref_file" && ! -d "$ref_file" ]]; then
                    echo -e "  ${RED}❌${NC} $ref_file - REFERENCED BUT NOT FOUND (in $file)"
                    ((ERRORS++))
                    ((ref_issues++))
                fi
            fi
        fi
    done < "$file"

    # Only show message if we actually checked references
    local checked_refs=false
    while IFS= read -r line; do
        if [[ "$line" =~ docs/[^[:space:]]+\.(txt|md) ]] || \
           [[ "$line" =~ \.eggs/[^[:space:]]+\.(txt|md) ]] || \
           [[ "$line" =~ bin/[^[:space:]]+\.sh ]]; then
            checked_refs=true
            break
        fi
    done < "$file"
}

# Check all files silently, only output problems
check_file "START.txt" "main entry point"
check_file "docs/procs/QUICK_REF.txt" "quick reference" "critical"
check_file "README.md" "project documentation"

check_dir "docs/procs" "process directory"
check_file "docs/procs/PROCESS.txt" "master workflow"
check_file "docs/procs/CONTINUE.md" "session status" "critical"
check_file "docs/procs/SPRINT.txt" "current sprint" "critical"
check_file "docs/procs/ROADMAP.txt" "strategic overview"
check_file "docs/procs/TASKS.txt" "detailed tasks"
check_file "docs/procs/DONE.txt" "completed work"

check_dir "docs/ref" "reference directory"
check_file "docs/ref/LESSONS.md" "development lessons"
check_file "docs/ref/DEVELOPMENT_NOTES.md" "session notes"

# Check analysis directory if it exists
if [[ -d ".eggs" ]]; then
    check_dir ".eggs" "analysis directory"
fi

check_file "bin/deploy.sh" "deployment script"
check_file "config.toml" "main configuration"
check_dir "examples" "examples directory"
check_dir "src" "source code directory"

# Check internal references
ref_issues=0
check_references_in_file "START.txt" "entry point references"
check_references_in_file "docs/procs/PROCESS.txt" "process references"
check_references_in_file "docs/procs/QUICK_REF.txt" "quick ref references"

# Only show status if there were issues
if [[ $ERRORS -gt 0 || $WARNINGS -gt 0 || $ref_issues -gt 0 ]]; then
    echo
    echo "=== ISSUES FOUND ==="
    if [[ $ref_issues -eq 0 ]]; then
        echo -e "${BLUE}Internal references: all valid${NC}"
    fi
fi

echo
echo "=== VALIDATION SUMMARY ==="
if [[ $ERRORS -eq 0 ]]; then
    # Silent success - no output when everything is good
    exit 0
else
    echo -e "${RED}❌ VALIDATION FAILED${NC}"
    echo -e "${RED}Errors found: $ERRORS${NC}"
    if [[ $WARNINGS -gt 0 ]]; then
        echo -e "${YELLOW}Warnings: $WARNINGS${NC}"
    fi
    echo
    echo "Please fix the missing files or update the references."
    exit 1
fi