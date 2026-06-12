#!/usr/bin/env python3
"""Clean up stubs.nr by removing implemented stub functions."""

import re

# Read the stubs file
with open('xpath/src/stubs.nr', 'r') as f:
    content = f.read()

# Stubs to remove (now implemented)
stubs_to_remove = [
    'stub_fncontains_token',
    'stub_fndata',
    'stub_fndefault_collation',
    'stub_fndefault_language',
    'stub_fnimplicit_timezone',
    'stub_fnqname',
    'stub_fnstring',
    'stub_fntokenize',
]

# Remove each stub function
removed_count = 0
for stub in stubs_to_remove:
    # Pattern matches the entire function definition
    pattern = rf'/// Stub for [^\n]*\n*pub fn {stub}\(\)[^\}}]*\}}\s*'
    new_content, count = re.subn(pattern, '', content)
    if count > 0:
        content = new_content
        removed_count += count
        print(f"Removed: {stub}")
    else:
        # Try simpler pattern
        pattern = rf'pub fn {stub}\(\)[^\}}]*\}}\s*'
        new_content, count = re.subn(pattern, '', content)
        if count > 0:
            content = new_content
            removed_count += count
            print(f"Removed: {stub}")

# Clean up multiple blank lines
content = re.sub(r'\n{3,}', '\n\n', content)

# Write the cleaned content
with open('xpath/src/stubs.nr', 'w') as f:
    f.write(content)

print(f'\nTotal removed: {removed_count} stub functions')
