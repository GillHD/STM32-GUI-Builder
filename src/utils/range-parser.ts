/**
 * Parses a string of comma-separated numbers and ranges (e.g., "11, 23-26, 30")
 * into an array of unique numbers within the specified min and max constraints.
 * @param rangeStr Input string (e.g., "11, 23-26, 30")
 * @param min Minimum allowed value
 * @param max Maximum allowed value
 * @returns Array of unique numbers in ascending order
 */
export function parseNumericRange(rangeStr: string | null | undefined, min: number, max: number): number[] {
  if (!rangeStr?.trim()) return [];

  const types: number[] = [];
  // Split by commas, trim each part, and filter out empty parts
  const parts = rangeStr.split(',').map(p => p.trim()).filter(p => p);

  for (const part of parts) {
    // Normalize range part to handle spaces around hyphen (e.g., "23 - 26" -> "23-26")
    const normalizedPart = part.replace(/\s*-\s*/, '-');

    if (normalizedPart.includes('-')) {
      const [start, end] = normalizedPart.split('-').map(n => parseInt(n.trim(), 10));
      if (!isNaN(start) && !isNaN(end) && start >= min && end <= max && start <= end) {
        for (let i = start; i <= end; i++) {
          if (!types.includes(i)) types.push(i);
        }
      }
    } else {
      const num = parseInt(part, 10);
      if (!isNaN(num) && num >= min && num <= max && !types.includes(num)) {
        types.push(num);
      }
    }
  }

  return types.sort((a, b) => a - b);
}

/**
 * Validates a string of comma-separated numbers and ranges against min and max constraints.
 * @param rangeStr Input string (e.g., "11, 23-26, 30")
 * @param min Minimum allowed value
 * @param max Maximum allowed value
 * @returns True if the string is valid, false otherwise
 */
export function validateNumericRange(rangeStr: string | null | undefined, min: number, max: number): boolean {
  if (!rangeStr?.trim()) return true;

  const trimmed = rangeStr.trim();
  // Allow commas, numbers, and ranges (e.g., "11,23-26,30")
  const validFormat = /^(\d+(-\d+)?)(,\s*\d+(-\d+)?)*$/;
  if (!validFormat.test(trimmed.replace(/\s*-\s*/g, '-'))) return false;

  return trimmed.split(',').every(part => {
    const partTrimmed = part.trim().replace(/\s*-\s*/, '-');
    if (partTrimmed.includes('-')) {
      const [start, end] = partTrimmed.split('-').map(n => parseInt(n.trim(), 10));
      return !isNaN(start) && !isNaN(end) && start >= min && end <= max && start <= end;
    }
    const num = parseInt(partTrimmed, 10);
    return !isNaN(num) && num >= min && num <= max;
  });
}

/**
 * Returns the minimum and maximum bounds of a numeric range.
 * @param numbers Array of numbers
 * @param min Minimum allowed value (used as default if numbers is empty)
 * @returns Object with start and end bounds
 */
export function numericRangeToBounds(numbers: number[], min: number): { start: number; end: number } {
  if (!numbers.length) return { start: min, end: min };
  return {
    start: Math.min(...numbers),
    end: Math.max(...numbers)
  };
}
