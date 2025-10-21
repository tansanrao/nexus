import type { PatchMetadata } from '@/types';

export interface PatchFoldRange {
  start: number;
  end: number;
}

export function getPatchFoldRange(metadata?: PatchMetadata | null): PatchFoldRange | null {
  if (!metadata || !metadata.diff_sections || metadata.diff_sections.length === 0) {
    return null;
  }

  let start = Number.MAX_SAFE_INTEGER;
  let end = Number.MIN_SAFE_INTEGER;

  metadata.diff_sections.forEach((section) => {
    start = Math.min(start, section.start_line);
    end = Math.max(end, section.end_line);
  });

  if (metadata.diffstat_section) {
    start = Math.min(start, metadata.diffstat_section.start_line);
    end = Math.max(end, metadata.diffstat_section.end_line);
  }

  metadata.trailer_sections.forEach((section) => {
    start = Math.min(start, section.start_line);
    end = Math.max(end, section.end_line);
  });

  if (typeof metadata.separator_line === 'number') {
    start = Math.min(start, metadata.separator_line);
    end = Math.max(end, metadata.separator_line);
  }

  if (!Number.isFinite(start) || !Number.isFinite(end) || start === Number.MAX_SAFE_INTEGER) {
    return null;
  }

  if (end < start) {
    return null;
  }

  return {
    start: Math.max(0, start),
    end: Math.max(start, end),
  };
}

export function buildPatchPreview(
  body: string,
  metadata?: PatchMetadata | null,
  fallbackLines = 3,
): string {
  const lines = body.split('\n');
  const fold = getPatchFoldRange(metadata);

  if (!fold) {
    return body;
  }

  const clampedStart = Math.max(0, Math.min(fold.start, lines.length));
  const previewLines = lines.slice(0, clampedStart);

  if (previewLines.length === 0) {
    return lines.slice(0, Math.min(fallbackLines, lines.length)).join('\n');
  }

  return previewLines.join('\n');
}
