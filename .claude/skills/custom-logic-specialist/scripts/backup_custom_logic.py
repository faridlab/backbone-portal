#!/usr/bin/env python3
"""
Custom Logic Backup Script
Safely backs up custom logic before schema regeneration
"""

import os
import sys
import shutil
import json
import argparse
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Set, Optional
import hashlib

class CustomLogicBackup:
    def __init__(self, module_path: str, backup_dir: str = None):
        self.module_path = Path(module_path)
        self.backup_dir = Path(backup_dir or f"/tmp/custom_logic_backup_{datetime.now().strftime('%Y%m%d_%H%M%S')}")
        self.custom_files: Dict[str, List[Path]] = {
            'services': [],
            'value_objects': [],
            'custom_sections': [],
            'tests': []
        }
        self.backup_metadata = {
            'timestamp': datetime.now().isoformat(),
            'module_path': str(self.module_path),
            'files_backed_up': {},
            'custom_sections_found': [],
            'checksums': {}
        }

    def discover_custom_logic(self) -> None:
        """Discover all custom logic files and sections"""
        print("🔍 Discovering custom logic...")

        # Custom domain services (100% custom)
        services_dir = self.module_path / 'src' / 'domain' / 'services'
        if services_dir.exists():
            for file_path in services_dir.glob('*.rs'):
                if not file_path.name.startswith('_') and file_path.is_file():
                    self.custom_files['services'].append(file_path)
                    print(f"  Found custom service: {file_path.relative_to(self.module_path)}")

        # Custom value objects (100% custom)
        vo_dir = self.module_path / 'src' / 'domain' / 'value_objects'
        if vo_dir.exists():
            for file_path in vo_dir.glob('*.rs'):
                if not file_path.name.startswith('_') and file_path.is_file():
                    self.custom_files['value_objects'].append(file_path)
                    print(f"  Found custom value object: {file_path.relative_to(self.module_path)}")

        # Test files
        for test_dir in [
            self.module_path / 'src' / 'domain' / 'services' / 'tests',
            self.module_path / 'src' / 'domain' / 'value_objects' / 'tests',
            self.module_path / 'tests'
        ]:
            if test_dir.exists():
                for file_path in test_dir.glob('**/*_test.rs'):
                    self.custom_files['tests'].append(file_path)
                    print(f"  Found test file: {file_path.relative_to(self.module_path)}")

        # Custom sections in generated files
        self.find_custom_sections()

    def find_custom_sections(self) -> None:
        """Find files with custom sections"""
        print("  Scanning for custom sections in generated files...")

        for root, dirs, files in os.walk(self.module_path):
            # Skip test directories and hidden files
            dirs[:] = [d for d in dirs if not d.startswith('.') and d != 'target']

            for file in files:
                if file.endswith('.rs'):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read()

                        if '// <<< CUSTOM' in content:
                            # Extract custom sections
                            custom_sections = self.extract_custom_sections(content)
                            if custom_sections:
                                self.custom_files['custom_sections'].append(file_path)
                                self.backup_metadata['custom_sections_found'].append({
                                    'file': str(file_path.relative_to(self.module_path)),
                                    'sections': list(custom_sections.keys())
                                })
                                print(f"    Custom sections in: {file_path.relative_to(self.module_path)}")
                    except (UnicodeDecodeError, IOError) as e:
                        print(f"    ⚠️  Could not read {file_path}: {e}")

    def extract_custom_sections(self, content: str) -> Dict[str, str]:
        """Extract custom sections from file content"""
        sections = {}

        # Find all custom section blocks
        import re
        pattern = r'// <<< CUSTOM (.+?) START >>>(.*?)// <<< CUSTOM \1 END >>>'
        matches = re.findall(pattern, content, re.DOTALL)

        for section_name, section_content in matches:
            sections[section_name.strip()] = section_content.strip()

        return sections

    def create_backup(self) -> None:
        """Create backup of all custom logic"""
        print(f"\n💾 Creating backup in {self.backup_dir}")
        self.backup_dir.mkdir(parents=True, exist_ok=True)

        # Backup custom files
        for category, files in self.custom_files.items():
            if files:
                category_dir = self.backup_dir / category
                category_dir.mkdir(exist_ok=True)

                for file_path in files:
                    relative_path = file_path.relative_to(self.module_path)
                    backup_path = category_dir / relative_path

                    # Create directory structure
                    backup_path.parent.mkdir(parents=True, exist_ok=True)

                    # Copy file
                    shutil.copy2(file_path, backup_path)

                    # Calculate checksum
                    checksum = self.calculate_checksum(file_path)
                    self.backup_metadata['files_backed_up'][str(relative_path)] = {
                        'category': category,
                        'checksum': checksum,
                        'size': file_path.stat().st_size,
                        'modified': datetime.fromtimestamp(file_path.stat().st_mtime).isoformat()
                    }

                    print(f"  Backed up: {relative_path}")

        # Save custom sections separately for easy restoration
        self.save_custom_sections()

        # Save metadata
        self.save_metadata()

    def save_custom_sections(self) -> None:
        """Save custom sections in a format easy to restore"""
        sections_dir = self.backup_dir / 'custom_sections_extracted'
        sections_dir.mkdir(exist_ok=True)

        for file_info in self.backup_metadata['custom_sections_found']:
            file_path = self.module_path / file_info['file']

            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()

                custom_sections = self.extract_custom_sections(content)

                # Save each section to its own file
                for section_name, section_content in custom_sections.items():
                    section_filename = f"{Path(file_info['file']).stem}_{section_name.replace(' ', '_').lower()}.rs"
                    section_file = sections_dir / section_filename

                    with open(section_file, 'w', encoding='utf-8') as f:
                        f.write(f"// Custom section: {section_name}\n")
                        f.write(f"// From file: {file_info['file']}\n")
                        f.write("// " + "=" * 60 + "\n")
                        f.write(section_content)
                        f.write("\n")

            except Exception as e:
                print(f"    ⚠️  Could not extract sections from {file_info['file']}: {e}")

    def save_metadata(self) -> None:
        """Save backup metadata"""
        metadata_file = self.backup_dir / 'backup_metadata.json'
        with open(metadata_file, 'w', encoding='utf-8') as f:
            json.dump(self.backup_metadata, f, indent=2, ensure_ascii=False)

        print(f"  Saved metadata: backup_metadata.json")

    def calculate_checksum(self, file_path: Path) -> str:
        """Calculate MD5 checksum of file"""
        hash_md5 = hashlib.md5()
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                hash_md5.update(chunk)
        return hash_md5.hexdigest()

    def verify_backup(self) -> bool:
        """Verify backup integrity"""
        print("\n✅ Verifying backup integrity...")

        all_valid = True

        for file_path_str, metadata in self.backup_metadata['files_backed_up'].items():
            backup_path = self.backup_dir / metadata['category'] / file_path_str

            if not backup_path.exists():
                print(f"  ❌ Missing backup file: {file_path_str}")
                all_valid = False
                continue

            # Verify checksum
            actual_checksum = self.calculate_checksum(backup_path)
            if actual_checksum != metadata['checksum']:
                print(f"  ❌ Checksum mismatch: {file_path_str}")
                all_valid = False
            else:
                print(f"  ✅ Verified: {file_path_str}")

        return all_valid

    def generate_restore_script(self) -> None:
        """Generate script to restore custom logic"""
        script_content = '''#!/bin/bash
# Custom Logic Restore Script
# Generated by backup_custom_logic.py

set -e

BACKUP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODULE_PATH="{module_path}"

echo "🔄 Restoring custom logic from $BACKUP_DIR"
echo "Target module: $MODULE_PATH"

# Function to restore file with backup
restore_file() {{
    local src="$1"
    local dst="$2"
    local backup_dir="$3"

    if [ -f "$dst" ]; then
        # Create backup of current file
        cp "$dst" "$dst.backup.$(date +%Y%m%d_%H%M%S)"
        echo "  📦 Backed up: $dst"
    fi

    # Ensure directory exists
    mkdir -p "$(dirname "$dst")"

    # Copy restored file
    cp "$src" "$dst"
    echo "  ✅ Restored: $dst"
}}

# Restore custom services
if [ -d "$BACKUP_DIR/services" ]; then
    echo "🔧 Restoring custom services..."
    find "$BACKUP_DIR/services" -name "*.rs" -type f | while read -r file; do
        rel_path="${file#$BACKUP_DIR/services/}"
        restore_file "$file" "$MODULE_PATH/src/domain/services/$rel_path" "$BACKUP_DIR"
    done
fi

# Restore custom value objects
if [ -d "$BACKUP_DIR/value_objects" ]; then
    echo "🏗️  Restoring custom value objects..."
    find "$BACKUP_DIR/value_objects" -name "*.rs" -type f | while read -r file; do
        rel_path="${file#$BACKUP_DIR/value_objects/}"
        restore_file "$file" "$MODULE_PATH/src/domain/value_objects/$rel_path" "$BACKUP_DIR"
    done
fi

# Restore custom sections (if separate)
if [ -d "$BACKUP_DIR/custom_sections_extracted" ]; then
    echo "📝 Restoring custom sections..."
    echo "⚠️  Manual restoration required for custom sections!"
    echo "   Review files in: $BACKUP_DIR/custom_sections_extracted"
fi

# Restore tests
if [ -d "$BACKUP_DIR/tests" ]; then
    echo "🧪 Restoring test files..."
    find "$BACKUP_DIR/tests" -name "*.rs" -type f | while read -r file; do
        rel_path="${file#$BACKUP_DIR/tests/}"
        restore_file "$file" "$MODULE_PATH/$rel_path" "$BACKUP_DIR"
    done
fi

echo ""
echo "✨ Custom logic restoration completed!"
echo ""
echo "Next steps:"
echo "1. Run 'cargo check' to verify compilation"
echo "2. Run 'cargo test' to verify functionality"
echo "3. Manually review and merge custom sections if needed"
echo "4. Commit changes to git"
'''.format(module_path=str(self.module_path))

        script_path = self.backup_dir / 'restore_custom_logic.sh'
        with open(script_path, 'w', encoding='utf-8') as f:
            f.write(script_content)

        # Make executable
        os.chmod(script_path, 0o755)
        print(f"  Generated restore script: restore_custom_logic.sh")

    def run(self) -> bool:
        """Run the complete backup process"""
        try:
            self.discover_custom_logic()
            self.create_backup()

            if self.verify_backup():
                self.generate_restore_script()

                print(f"\n✨ Backup completed successfully!")
                print(f"📁 Backup location: {self.backup_dir}")
                print(f"📊 Files backed up: {len(self.backup_metadata['files_backed_up'])}")
                print(f"📝 Custom sections: {len(self.backup_metadata['custom_sections_found'])}")
                print(f"\nTo restore, run: {self.backup_dir}/restore_custom_logic.sh")

                return True
            else:
                print("\n❌ Backup verification failed!")
                return False

        except Exception as e:
            print(f"\n❌ Backup failed: {e}")
            return False


def main():
    parser = argparse.ArgumentParser(description='Backup custom logic before schema regeneration')
    parser.add_argument('module_path', help='Path to the module (e.g., libs/modules/sapiens)')
    parser.add_argument('--backup-dir', help='Custom backup directory')
    parser.add_argument('--verify-only', action='store_true', help='Only verify existing backup')

    args = parser.parse_args()

    if args.verify_only:
        # Verify existing backup
        if not args.backup_dir:
            print("❌ --backup-dir required when using --verify-only")
            sys.exit(1)

        backup_dir = Path(args.backup_dir)
        if not backup_dir.exists():
            print(f"❌ Backup directory not found: {backup_dir}")
            sys.exit(1)

        # Load metadata and verify
        metadata_file = backup_dir / 'backup_metadata.json'
        if not metadata_file.exists():
            print(f"❌ Backup metadata not found: {metadata_file}")
            sys.exit(1)

        with open(metadata_file, 'r') as f:
            metadata = json.load(f)

        print(f"🔍 Verifying backup from: {backup_dir}")
        print(f"📅 Backup created: {metadata['timestamp']}")

        # Check files exist
        missing_files = []
        for file_path in metadata['files_backed_up']:
            backup_file = backup_dir / metadata['files_backed_up'][file_path]['category'] / file_path
            if not backup_file.exists():
                missing_files.append(file_path)

        if missing_files:
            print(f"❌ Missing files in backup: {missing_files}")
            sys.exit(1)
        else:
            print("✅ All files present in backup")
            sys.exit(0)
    else:
        # Create new backup
        backup = CustomLogicBackup(args.module_path, args.backup_dir)
        success = backup.run()
        sys.exit(0 if success else 1)


if __name__ == '__main__':
    main()