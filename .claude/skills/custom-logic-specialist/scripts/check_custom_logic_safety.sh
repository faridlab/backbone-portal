#!/bin/bash

# Custom Logic Safety Check Script
# Validates that custom logic is safely implemented within Backbone Framework
# Usage: ./check_custom_logic_safety.sh [module_name]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MODULE_NAME="${1:-sapiens}"
MODULE_PATH="libs/modules/$MODULE_NAME"
MAX_WARNINGS=10

# Counters
WARNINGS=0
ERRORS=0

echo -e "${BLUE}🔍 Custom Logic Safety Check for $MODULE_NAME${NC}"
echo "=================================================="

# Function to print section header
print_section() {
    echo -e "\n${BLUE}$1${NC}"
    echo "$(printf '=%.0s' {1..50})"
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠️  WARNING: $1${NC}"
    ((WARNINGS++))
}

# Function to print error
print_error() {
    echo -e "${RED}❌ ERROR: $1${NC}"
    ((ERRORS++))
}

# Function to print success
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function to print info
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if module exists
if [ ! -d "$MODULE_PATH" ]; then
    print_error "Module $MODULE_NAME not found at $MODULE_PATH"
    exit 1
fi

print_section "📁 Checking Directory Structure"

# Check for custom service directories
if [ -d "$MODULE_PATH/src/domain/services" ]; then
    service_count=$(find "$MODULE_PATH/src/domain/services" -name "*.rs" -not -path "*/tests/*" | wc -l)
    print_success "Found $service_count custom domain service(s)"
else
    print_warning "No custom domain services directory found"
fi

# Check for custom value objects
if [ -d "$MODULE_PATH/src/domain/value_objects" ]; then
    vo_count=$(find "$MODULE_PATH/src/domain/value_objects" -name "*.rs" -not -path "*/tests/*" | wc -l)
    print_success "Found $vo_count custom value object(s)"
else
    print_warning "No custom value objects directory found"
fi

print_section "🔒 Checking Generated File Modifications"

# Find generated files that might have been modified unsafely
GENERATED_PATTERNS=(
    "src/application/service/*_service.rs"
    "src/domain/entities/*"
    "src/infrastructure/persistence/*"
    "src/presentation/handlers/*_handler.rs"
    "proto/*"
)

for pattern in "${GENERATED_PATTERNS[@]}"; do
    if [ -f "$MODULE_PATH/$pattern" ] && [ -d "$MODULE_PATH/$(dirname "$pattern")" ]; then
        # Check for modifications outside custom sections
        if grep -q "// <<< CUSTOM" "$MODULE_PATH/$pattern" 2>/dev/null; then
            # File has custom sections - check if there are modifications outside them
            if grep -v -A 1000 "// <<< CUSTOM.*START >>>" "$MODULE_PATH/$pattern" | \
               grep -v -B 1000 "// <<< CUSTOM.*END >>>" | \
               grep -v "// <<< CUSTOM" | \
               grep -v "^//" | \
               grep -v "^[[:space:]]*$" | \
               grep -q "pub.*{" 2>/dev/null; then
                print_warning "Possible unsafe modifications in $pattern"
            else
                print_success "Custom logic safely placed in $pattern"
            fi
        else
            # Check if file has been modified at all
            if [ -f "$MODULE_PATH/$pattern" ]; then
                print_info "Review $pattern - should be generated only"
            fi
        fi
    fi
done

print_section "📝 Checking Custom Section Patterns"

# Check for properly formatted custom sections
CUSTOM_SECTION_FILES=$(find "$MODULE_PATH" -name "*.rs" -exec grep -l "// <<< CUSTOM" {} \;)

for file in $CUSTOM_SECTION_FILES; do
    # Check if custom sections are properly paired
    start_sections=$(grep -c "// <<< CUSTOM.*START >>>" "$file" || true)
    end_sections=$(grep -c "// <<< CUSTOM.*END >>>" "$file" || true)

    if [ "$start_sections" -eq "$end_sections" ]; then
        print_success "Custom sections properly paired in $(basename "$file")"
    else
        print_error "Mismatched custom sections in $(basename "$file"): $start_sections start, $end_sections end"
    fi
done

print_section "🧪 Checking Test Coverage"

# Check for custom logic tests
if [ -d "$MODULE_PATH/src/domain/services/tests" ]; then
    test_count=$(find "$MODULE_PATH/src/domain/services/tests" -name "*_test.rs" | wc -l)
    print_success "Found $test_count custom service test file(s)"
else
    print_warning "No tests found for custom services"
fi

# Check for test coverage in value objects
if [ -d "$MODULE_PATH/src/domain/value_objects" ]; then
    vo_with_tests=$(find "$MODULE_PATH/src/domain/value_objects" -name "*_test.rs" | wc -l)
    if [ "$vo_with_tests" -gt 0 ]; then
        print_success "Found $vo_with_tests value object test file(s)"
    else
        print_warning "No tests found for custom value objects"
    fi
fi

print_section "🔗 Checking Dependencies"

# Check for circular dependencies in custom services
if [ -f "$MODULE_PATH/Cargo.toml" ]; then
    # Check for potentially problematic dependencies
    if grep -q "tokio.*features.*full" "$MODULE_PATH/Cargo.toml"; then
        print_warning "Using tokio with full features - consider specific features"
    fi

    if grep -q "log.*features.*max_level" "$MODULE_PATH/Cargo.toml"; then
        print_warning "Log level configured in Cargo.toml - consider runtime configuration"
    fi
fi

print_section "📊 Analyzing Code Metrics"

# Check function sizes in custom services
if [ -d "$MODULE_PATH/src/domain/services" ]; then
    for service_file in $MODULE_PATH/src/domain/services/*.rs; do
        if [ -f "$service_file" ] && [[ ! "$service_file" =~ tests ]]; then
            # Count lines in functions (simplified check)
            long_functions=$(awk '
                /^impl/ { in_impl = 1 }
                /^}/ && in_impl { in_impl = 0 }
                in_impl && /pub async fn/ {
                    func_start = NR
                    func_name = $4
                    brace_count = 1
                    next
                }
                in_impl && func_start && /\{/ {
                    brace_count += gsub(/\{/, "", $0)
                    brace_count -= gsub(/\}/, "", $0)
                    if (brace_count == 0) {
                        lines = NR - func_start
                        if (lines > 50) print func_name ": " lines " lines"
                        func_start = 0
                    }
                }
            ' "$service_file")

            if [ -n "$long_functions" ]; then
                print_warning "Long functions in $(basename "$service_file"):"
                echo "$long_functions" | sed 's/^/   /'
            fi
        fi
    done
fi

# Check for complex business logic in handlers
HANDLER_FILES=$(find "$MODULE_PATH/src/presentation/handlers" -name "*_handler.rs" 2>/dev/null || true)
for handler_file in $HANDLER_FILES; do
    business_logic_indicators=$(grep -c "if.*&&.*||" "$handler_file" 2>/dev/null || true)
    if [ "$business_logic_indicators" -gt 5 ]; then
        print_warning "Complex business logic detected in $(basename "$handler_file") - consider moving to domain service"
    fi
done

print_section "🔍 Security Checks"

# Check for hardcoded secrets or credentials
SECRET_PATTERNS=(
    "password.*=.*\""
    "secret.*=.*\""
    "key.*=.*\".*[A-Za-z0-9]{20,}"
    "token.*=.*\".*[A-Za-z0-9]{20,}"
)

for pattern in "${SECRET_PATTERNS[@]}"; do
    matches=$(find "$MODULE_PATH/src" -name "*.rs" -exec grep -l "$pattern" {} \; 2>/dev/null || true)
    if [ -n "$matches" ]; then
        print_error "Potential hardcoded secrets found:"
        echo "$matches" | sed 's/^/   /'
    fi
done

# Check for SQL injection vulnerabilities
sql_patterns=(
    "format!.*SELECT"
    "format!.*INSERT"
    "format!.*UPDATE"
    "format!.*DELETE"
    "\.query("
)

for pattern in "${sql_patterns[@]}"; do
    matches=$(find "$MODULE_PATH/src" -name "*.rs" -exec grep -l "$pattern" {} \; 2>/dev/null || true)
    if [ -n "$matches" ]; then
        print_warning "Potential SQL injection vulnerability in:"
        echo "$matches" | sed 's/^/   /'
    fi
done

print_section "🚀 Performance Checks"

# Check for async functions without proper error handling
async_no_result=$(find "$MODULE_PATH/src/domain/services" -name "*.rs" -exec grep -l "async fn.*-> ()" {} \; 2>/dev/null || true)
if [ -n "$async_no_result" ]; then
    print_warning "Async functions without Result return type in:"
    echo "$async_no_result" | sed 's/^/   /'
fi

# Check for potential memory leaks
arc_clones=$(find "$MODULE_PATH/src" -name "*.rs" -exec grep -c "\.clone()" {} \; 2>/dev/null | awk -F: '$2 > 10 { print $1 }')
if [ -n "$arc_clones" ]; then
    print_warning "High clone count in:"
    echo "$arc_clones" | sed 's/^/   /'
fi

print_section "📋 Summary"

echo -e "\n${BLUE}Safety Check Results:${NC}"
echo "===================="
echo -e "Warnings: ${YELLOW}$WARNINGS${NC}"
echo -e "Errors: ${RED}$ERRORS${NC}"

if [ $ERRORS -eq 0 ] && [ $WARNINGS -le $MAX_WARNINGS ]; then
    print_success "✨ Custom logic appears to be safely implemented!"
    echo -e "${GREEN}No critical issues detected. Your custom logic is ready for production.${NC}"
    exit_code=0
elif [ $ERRORS -eq 0 ]; then
    print_warning "Some warnings detected. Review and address before production."
    exit_code=1
else
    print_error "Critical issues found. Fix before proceeding with regeneration."
    exit_code=2
fi

# Recommendations
echo -e "\n${BLUE}Recommendations:${NC}"
echo "================"
if [ $ERRORS -gt 0 ]; then
    echo "1. Fix all critical errors before regeneration"
    echo "2. Move complex business logic from handlers to domain services"
    echo "3. Ensure proper error handling in async functions"
fi
if [ $WARNINGS -gt 0 ]; then
    echo "4. Add comprehensive tests for custom business logic"
    echo "5. Consider breaking down large functions into smaller ones"
    echo "6. Review and optimize performance bottlenecks"
fi
echo "7. Always test after schema regeneration"
echo "8. Keep documentation updated with business rule changes"

exit $exit_code