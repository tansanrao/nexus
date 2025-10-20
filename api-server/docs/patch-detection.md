# Patch Detection Metadata

## Schema Changes
- Introduced `patch_type` enum with values `none`, `inline`, `attachment`.
- Added `patch_type`, `is_patch_only`, `patch_metadata` columns and supporting indexes on `emails`.

## Model Updates
- `EmailWithAuthor` and `EmailHierarchy` expose patch metadata; custom `FromRow` handles JSON.

## Parsing Logic
- Inline detection identifies diff headers, diffstat, trailers, and sets `is_patch_only`.
- Attachment detection inspects MIME types, filenames, and disposition.

## Import Pipeline
- `EmailsData` carries `patch_types`, `is_patch_only`, `patch_metadata`; bulk insert writes them.

## API Exposure
- Email, author, and thread endpoints now return patch metadata to clients.

## Testing
- Added parser unit tests for inline patches, tail text, and cover letters (`cargo test sync::parser`).
