# Custom Logic Specialist

This Backbone Framework skill teaches developers how to safely add custom business logic within the framework while maintaining compatibility with code generation.

## Overview

The Custom Logic Specialist provides comprehensive guidance on extending Backbone modules with sophisticated business capabilities without risking regeneration conflicts or compromising framework integrity.

## What You'll Learn

1. **Safe Extension Patterns** - Master the `// <<< CUSTOM` section pattern for generated files
2. **Custom Architecture Design** - Create custom services, value objects, and handlers
3. **Framework Integration** - Ensure custom logic integrates seamlessly with Backbone patterns
4. **Testing Strategies** - Comprehensive testing patterns for custom business logic
5. **Regeneration Safety** - Guidelines for safe schema regeneration
6. **Common Pitfalls** - How to avoid dangerous mistakes

## Directory Structure

```
.custom-logic-specialist/
├── skill.md                    # Main skill documentation
├── templates/                  # Templates for custom components
│   ├── custom_domain_service_template.md
│   ├── custom_value_object_template.md
│   ├── custom_handler_template.md
│   └── custom_service_test_template.md
├── examples/                   # Real-world implementation examples
│   └── real_world_examples.md
├── scripts/                    # Utility scripts for safety checks
│   ├── check_custom_logic_safety.sh
│   └── backup_custom_logic.py
└── README.md                   # This file
```

## Quick Start

### 1. Understand Safe vs Unsafe Modifications

```bash
# Check if your custom logic is safely implemented
./scripts/check_custom_logic_safety.sh sapiens
```

### 2. Backup Before Regeneration

```bash
# Backup your custom logic before schema changes
python3 scripts/backup_custom_logic.py libs/modules/sapiens
```

### 3. Use Templates for New Custom Logic

- **Custom Domain Service**: Use `templates/custom_domain_service_template.md`
- **Custom Value Objects**: Use `templates/custom_value_object_template.md`
- **Custom Handlers**: Use `templates/custom_handler_template.md`

## Key Concepts

### ✅ SAFE TO MODIFY (100% Custom)

```rust
// Custom domain services - NEVER regenerated
src/domain/services/my_custom_service.rs

// Custom value objects - NEVER regenerated
src/domain/value_objects/my_custom_vo.rs

// Custom sections in generated files - PRESERVED during regeneration
// <<< CUSTOM COMMANDS START >>>
// Your custom code here
// <<< CUSTOM COMMANDS END >>>
```

### ❌ UNSAFE TO MODIFY (Generated)

```rust
// Generated service methods - WILL BE OVERWRITTEN
src/application/service/user_service.rs

// Generated entity structs - WILL BE OVERWRITTEN
src/domain/entities/user.rs

// Generated handler methods - WILL BE OVERWRITTEN
src/presentation/handlers/user_handler.rs
```

## Real-World Examples

### 1. E-Commerce Pricing Service
Complex pricing logic with:
- Volume discounts
- Customer tier pricing
- Seasonal promotions
- Tax calculations

### 2. Healthcare Eligibility Service
Insurance eligibility checking with:
- Coverage validation
- Medical necessity assessment
- Prior authorization requirements
- Cost estimation

### 3. Financial Fraud Detection
Real-time fraud analysis with:
- Pattern recognition
- Behavioral analysis
- Risk scoring
- Transaction monitoring

## Best Practices

### DO ✅
1. **Use Custom Domain Services** for complex business logic
2. **Create Custom Value Objects** for domain-specific types
3. **Leverage // <<< CUSTOM Sections** for extending generated code
4. **Write Comprehensive Tests** for all custom logic
5. **Backup Before Regeneration** using provided scripts
6. **Follow Framework Dependency Patterns**

### DON'T ❌
1. **Edit Generated Code** outside custom sections
2. **Add Methods to Generated Services** directly
3. **Hardcode Business Logic** in handlers
4. **Bypass Framework Infrastructure**
5. **Skip Testing** custom business logic
6. **Ignore Regeneration Safety** guidelines

## Using the Skill

To activate this skill and get expert guidance on custom logic implementation:

```bash
# Use the Custom Logic Specialist skill
Skill: custom-logic-specialist
```

### Example Questions to Ask

- "I need to add complex pricing logic to my order system. How should I structure this?"
- "I want to create a custom value object for email addresses with validation. What's the best approach?"
- "How can I extend my generated handlers without risking regeneration?"
- "What testing patterns should I use for my custom domain services?"
- "Is it safe to modify this generated file? How can I tell?"

## Scripts and Tools

### Safety Check Script
```bash
# Analyze your custom logic for safety issues
./scripts/check_custom_logic_safety.sh [module_name]

# Features:
- Detects unsafe modifications to generated files
- Checks for proper custom section formatting
- Analyzes code quality and complexity
- Identifies potential security issues
- Provides recommendations for improvement
```

### Backup Script
```bash
# Backup custom logic before regeneration
python3 scripts/backup_custom_logic.py [module_path] [--backup-dir /path/to/backup]

# Features:
- Identifies all custom logic files
- Extracts custom sections from generated files
- Creates comprehensive backup with metadata
- Generates restore script
- Verifies backup integrity
```

## Integration with Other Skills

Works closely with:
- **Backbone Schema Maintainer**: For understanding generated code structure
- **Creative Domain Architect**: For domain modeling decisions
- **Tests Maintainer**: For testing custom business logic
- **Database Migration Specialist**: When custom logic needs schema changes

## Contributing

To improve this skill:
1. Add new templates for common patterns
2. Enhance safety check scripts with more rules
3. Add real-world examples from your projects
4. Improve documentation and examples

## Support

For questions or issues with custom logic implementation:
1. Use the skill: `Skill: custom-logic-specialist`
2. Review the templates and examples
3. Run the safety check script for guidance
4. Check the Backbone Framework documentation

---

**Remember**: Custom logic should enhance the framework, not fight against it. Stay within the patterns and your code will be maintainable, safe, and future-proof!