#!/bin/bash

# Use ast-grep to add JsonSchema to structs/enums that have Serialize/Deserialize
# but don't have JsonSchema yet.

echo "🚀 Injecting JsonSchema derives using ast-grep..."

# Function to process files
process_pattern() {
    local pattern=$1
    local rewrite=$2
    
    ast-grep run \
        --pattern "$pattern" \
        --rewrite "$rewrite" \
        --lang rust \
        --update-all \
        familiar-core/src
}

# 1. Add to structs with Serialize/Deserialize in any order
# Structs with { ... }
process_pattern '#[derive($$$SERDE1, Serialize, $$$SERDE2)] struct $NAME { $$$FIELDS }' '#[derive($$$SERDE1, Serialize, $$$SERDE2, JsonSchema)] struct $NAME { $$$FIELDS }'
process_pattern '#[derive($$$SERDE1, Deserialize, $$$SERDE2)] struct $NAME { $$$FIELDS }' '#[derive($$$SERDE1, Deserialize, $$$SERDE2, JsonSchema)] struct $NAME { $$$FIELDS }'

# Unit structs
process_pattern '#[derive($$$SERDE1, Serialize, $$$SERDE2)] struct $NAME;' '#[derive($$$SERDE1, Serialize, $$$SERDE2, JsonSchema)] struct $NAME;'
process_pattern '#[derive($$$SERDE1, Deserialize, $$$SERDE2)] struct $NAME;' '#[derive($$$SERDE1, Deserialize, $$$SERDE2, JsonSchema)] struct $NAME;'

# Tuple structs
process_pattern '#[derive($$$SERDE1, Serialize, $$$SERDE2)] struct $NAME($$$TYPES);' '#[derive($$$SERDE1, Serialize, $$$SERDE2, JsonSchema)] struct $NAME($$$TYPES);'
process_pattern '#[derive($$$SERDE1, Deserialize, $$$SERDE2)] struct $NAME($$$TYPES);' '#[derive($$$SERDE1, Deserialize, $$$SERDE2, JsonSchema)] struct $NAME($$$TYPES);'

# 2. Add to enums
process_pattern '#[derive($$$SERDE1, Serialize, $$$SERDE2)] enum $NAME { $$$VARIANTS }' '#[derive($$$SERDE1, Serialize, $$$SERDE2, JsonSchema)] enum $NAME { $$$VARIANTS }'
process_pattern '#[derive($$$SERDE1, Deserialize, $$$SERDE2)] enum $NAME { $$$VARIANTS }' '#[derive($$$SERDE1, Deserialize, $$$SERDE2, JsonSchema)] enum $NAME { $$$VARIANTS }'

# 3. Clean up duplicates (if any)
# TODO: ast-grep doesn't easily support "does not contain" in patterns, 
# so we might have some "JsonSchema, JsonSchema". Let's fix that.
find familiar-core/src -name "*.rs" -exec sed -i '' 's/JsonSchema, JsonSchema/JsonSchema/g' {} +

# 4. Add imports
echo "📦 Adding schemars imports..."
find familiar-core/src -name "*.rs" -exec grep -l "JsonSchema" {} + | xargs -I {} sh -c 'grep -q "use schemars::JsonSchema;" {} || sed -i "" "/use serde/a\\
use schemars::JsonSchema;" {}'

echo "✅ Done!"





