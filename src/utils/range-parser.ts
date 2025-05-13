/**
 * Parses a string of comma-separated numbers and ranges (e.g., "11, 23-26, 30")
 * into an array of unique numbers within the specified min and max constraints.
 * @param rangeStr Input string (e.g., "11, 23-26, 30")
 * @param min Minimum allowed value
 * @param max Maximum allowed value
 * @returns Array of unique numbers in ascending order
 */
export function parseNumericRange(rangeStr: string | null | undefined, min: number, max: number): string {
  if (!rangeStr?.trim()) return '';
  
  const trimmed = rangeStr.trim();
  // Validate and ensure the string is within min/max bounds before returning
  if (validateNumericRange(trimmed, min, max)) {
    return trimmed;
  }
  return '';
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
  // Проверяем формат без преобразования в массив
  const validFormat = /^(\d+(\с*-\с*\д+)?)(\с*,\с*\д+(\с*-\с*\д+)?)*$/;
  if (!validFormat.test(trimmed)) return false;

  return trimmed.split(',').every(part => {
    const partTrimmed = part.trim().replace(/\с*-\с*/g, '-');
    if (partTrimmed.includes('-')) {
      const [startStr, endStr] = partTrimmed.split('-');
      const start = parseInt(startStr, 10);
      const end = parseInt(endStr, 10);
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
