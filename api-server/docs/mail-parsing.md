# Mail Parsing Pipeline

This document captures how the API server turns raw RFC 822 blobs into the
`ParsedEmail` structure. The implementation intentionally mirrors the parsing
behaviour of the [b4 patch tool](https://github.com/mricon/b4) so that patch
detection in the UI matches what we apply during maintenance.

## Body Extraction
- All MIME parts are walked depth-first (matching Python's `email.walk()`).
- Only `text/plain` and `text/x-patch` parts are considered as candidates.
- The first decodable candidate becomes the baseline body.
- If a later candidate contains an inline diff according to b4's regex (see
  below), it replaces the baseline. This favours the same payload b4 would use
  when applying a patch in multipart messages.
- A fallback call to `ParsedMail::get_body` keeps behaviour stable for the few
  messages without a plain-text part.

Implementation reference: `extract_preferred_body` in
`api-server/src/sync/parser.rs`.

## Inline Patch Detection
- Detection uses the exact regular expressions shipped with b4:
  - Diff hunk regex: `(?mi)^(---.*\n\+\+\+|GIT binary patch|diff --git \w/\S+ \w/\S+)`
  - Diffstat regex: `(?mi)^\s*\d+ file.*\d+ (insertion|deletion)`
- When the diff regex is present in the selected body we classify the email as
  `PatchType::Inline`.
- Metadata extraction (`analyze_inline_patch`) still computes diff sections,
  separators, trailers, and the optional diffstat block. The diffstat regex is
  used as a guard so we only mark a diffstat section when b4 would make the
  same inference.
- If metadata extraction fails (for example due to an unusual format) we still
  keep the inline classification but omit metadata.

## Attachment-driven Patches
- If no inline diff is present we scan all MIME parts for patch attachments
  (`text/x-diff`, `text/x-patch`, or filenames ending in `.patch`/`.diff`).
- We classify such messages as `PatchType::Attachment`. A simple heuristic
  checks whether the visible body contains quotes or additional prose to set
  `is_patch_only`.

## Metadata Summary
- `PatchMetadata` stores structured offsets for diff hunks, diffstat, trailers,
  and footers to power API consumers.
- Trailer detection is shared between inline and attachment classification so
  downstream systems can find sign-off acknowledgements even in follow-ups.

## Keeping Behaviour in Sync with b4
- Any future changes to b4's diff or diffstat regexes should be mirrored here.
- Our unit tests (`cargo test sync::parser`) cover inline, attachment, and
  cover-letter scenarios. Add new tests alongside changes to the parsing logic.
