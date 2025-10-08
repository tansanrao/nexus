# Email Threading Algorithm Improvements

## Summary

This document describes the improvements made to the email parsing and threading algorithms based on comparison with the [egol/mailing-list-parser](https://github.com/egol/mailing-list-parser) implementation.

## Problems Identified

### 1. **Limited References Parsing**
- **Old approach**: Character-by-character state machine parsing
- **Problem**: Overly complex, could miss Message-IDs not in strict `<id>` format
- **Impact**: Some email references were missed, leading to fragmented threads

### 2. **No Subject-Based Fallback**
- **Old approach**: Only used References and In-Reply-To headers
- **Problem**: LKML emails often have broken or missing headers
- **Impact**: Related emails weren't grouped into threads even when clearly related

### 3. **No Subject Normalization**
- **Old approach**: Stored and compared raw subjects
- **Problem**: "Re:", "[PATCH v2]", "[PATCH v3]" prevented proper matching
- **Impact**: Different versions of same patch series appeared as separate threads

### 4. **No Patch Series Support**
- **Old approach**: No special handling for patch series
- **Problem**: Patch series like "[PATCH 0/5]", "[PATCH 1/5]" weren't linked
- **Impact**: Kernel patch sets appeared fragmented instead of grouped

### 5. **No In-Reply-To Fallback**
- **Old approach**: In-Reply-To was stored but not used for threading
- **Problem**: Emails with In-Reply-To but broken References weren't threaded
- **Impact**: Reply chains were broken unnecessarily

## Improvements Made

### 1. Simplified References Parsing
**File**: `api-server/src/sync/parser.rs`

```rust
// OLD: Complex character-by-character parsing
fn extract_references(header_value: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut current = String::new();
    let mut in_brackets = false;
    for ch in header_value.chars() {
        match ch {
            '<' => { in_brackets = true; current.clear(); }
            '>' => { /* ... */ }
            _ => { /* ... */ }
        }
    }
    refs
}

// NEW: Simple whitespace-based splitting
fn extract_references(header_value: &str) -> Vec<String> {
    header_value
        .split_whitespace()
        .map(|id| {
            let cleaned = id.trim().trim_matches(&['<', '>'][..]);
            sanitize_text(cleaned)
        })
        .filter(|id| !id.is_empty())
        .collect()
}
```

**Benefit**: More robust, handles various Message-ID formats, simpler code.

### 2. Subject Normalization
**File**: `api-server/src/sync/parser.rs`

Added `normalize_subject()` function that:
- Removes "Re:", "Fwd:", "Fw:", "Aw:" prefixes
- Strips "[PATCH]", "[RFC]", version numbers, patch numbers
- Collapses multiple spaces
- Converts to lowercase for comparison

**Example**:
```
"Re: [PATCH v2 3/10] Fix memory leak" → "fix memory leak"
```

**Benefit**: Different versions/replies of same topic now match.

### 3. Multi-Strategy Threading Algorithm
**File**: `api-server/src/sync/importer.rs`

Enhanced the JWZ algorithm with **three fallback strategies**:

#### Strategy 1: In-Reply-To Fallback
- After References-based threading, check In-Reply-To for orphaned emails
- Links emails that have In-Reply-To but no References

#### Strategy 2: Subject-Based Matching
- Groups emails with same normalized subject
- Links orphaned emails to earliest email with matching subject
- Only applies if email still has no parent after previous strategies

#### Strategy 3: Patch Series Detection
- Detects patch series patterns: `[PATCH 2/5]`, `[PATCH v2 3/10]`
- Links patches in series to cover letter (0/N) or previous patch
- Handles versioned series separately

**Benefit**: Significantly reduces fragmented threads, especially for kernel patches.

### 4. Enhanced Data Model
**File**: `api-server/src/sync/parser.rs`

Added to `ParsedEmail` struct:
```rust
pub normalized_subject: String, // For threading fallback
```

**File**: `api-server/src/sync/importer.rs`

Enhanced `EmailData` struct:
```rust
struct EmailData {
    // ... existing fields
    normalized_subject: String,
    in_reply_to: Option<String>,
}
```

**Benefit**: Enables subject-based matching without re-parsing.

## Algorithm Flow

```
1. JWZ Algorithm (References-based)
   ├─ Create containers for all emails and references
   ├─ Build parent-child links from References chains
   └─ Identify root messages (no parent)

2. Fallback Strategy 1: In-Reply-To
   ├─ For emails without parent
   └─ Link to In-Reply-To if it exists

3. Fallback Strategy 2: Subject Matching
   ├─ For emails still without parent
   ├─ Find emails with same normalized subject
   └─ Link to earliest matching email

4. Fallback Strategy 3: Patch Series
   ├─ Detect patch series patterns
   ├─ Group by series (subject + version)
   └─ Link to cover letter (0/N) or sequential patches

5. Thread Assembly
   ├─ Identify root set
   ├─ Promote children of phantom roots
   └─ Insert threads and memberships
```

## Testing

All tests pass:
```bash
test sync::parser::tests::test_extract_references ... ok
test sync::parser::tests::test_clean_message_id ... ok
test sync::parser::tests::test_normalize_subject ... ok
test sync::parser::tests::test_sanitize_text ... ok
```

## Expected Improvements

1. **Fewer fragmented threads**: Subject-based matching will group related emails
2. **Better patch series handling**: Kernel patches will appear as unified threads
3. **More robust parsing**: Simplified References parsing handles edge cases better
4. **Backward compatibility**: All existing functionality preserved, only enhancements added

## Configuration

No configuration changes needed. Improvements are automatic.

## Dependencies Added

- `regex = "1.10"` - For patch series pattern matching

## Files Modified

1. `api-server/Cargo.toml` - Added regex dependency
2. `api-server/src/sync/parser.rs` - Subject normalization, simplified References parsing
3. `api-server/src/sync/importer.rs` - Multi-strategy threading algorithm

## Migration Notes

Existing databases will need to be rebuilt to take advantage of the new threading:
```bash
# Re-run the database population
cd poc
jupyter notebook poc.ipynb
# Or use the Rust importer
```

## References

- [JWZ Threading Algorithm](https://www.jwz.org/doc/threading.html)
- [egol/mailing-list-parser](https://github.com/egol/mailing-list-parser)
- [RFC 5322 - Internet Message Format](https://tools.ietf.org/html/rfc5322)
