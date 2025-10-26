import type { PatchMetadata } from '../types';

/**
 * Extracts diff content from an email body using optional patch metadata to limit sections.
 */
export function extractDiffContent(
  emailBody: string | null,
  patchMetadata: PatchMetadata | null
): string {
  if (!emailBody) {
    return '';
  }

  const lines = emailBody.split('\n');

  if (patchMetadata && patchMetadata.diff_sections.length > 0) {
    const diffLines: string[] = [];

    for (const section of patchMetadata.diff_sections) {
      for (let i = section.start_line; i <= section.end_line; i++) {
        if (i >= 0 && i < lines.length) {
          diffLines.push(lines[i]);
        }
      }
    }

    return diffLines.join('\n');
  }

  const diffStartPattern = /^diff --git|^---|^\+\+\+|^@@/;
  const diffLines: string[] = [];
  let inDiffSection = false;

  for (const line of lines) {
    if (diffStartPattern.test(line)) {
      inDiffSection = true;
    }

    if (inDiffSection) {
      diffLines.push(line);
    }
  }

  return diffLines.join('\n');
}
