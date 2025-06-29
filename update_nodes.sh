#!/bin/bash

# Script to update NodeMetadata struct definitions to use the new format

# Function to update a single file
update_node_file() {
    local file="$1"
    echo "Updating $file..."
    
    # Use sed to replace the old struct format with new() calls
    # This is a basic transformation - may need manual cleanup for complex cases
    sed -i '' '
        # Replace simple NodeMetadata { ... } with NodeMetadata::new(...) pattern
        /NodeMetadata {/{
            :loop
            N
            /}/!b loop
            # Now we have the full NodeMetadata block
            s/NodeMetadata {[^}]*node_type: "\([^"]*\)"[^}]*display_name: "\([^"]*\)"[^}]*category: \([^,]*\)[^}]*description: "\([^"]*\)"[^}]*}/NodeMetadata::new("\1", "\2", \3, "\4")/g
        }
    ' "$file"
}

# Find all Rust files with NodeMetadata definitions and update them
find src -name "*.rs" -exec grep -l "NodeMetadata {" {} \; | while read file; do
    # Skip the factory.rs file since we already updated it manually
    if [[ "$file" != "src/nodes/factory.rs" ]]; then
        update_node_file "$file"
    fi
done

echo "Bulk update complete. Manual review needed for complex cases."