export function overlayPosition(center: boolean, x: number | string, y: number | string) {
  function coordinate(value: number | string): number | null {
    if (typeof value === 'string' && !value.trim()) return null;
    const result = Number(value);
    return Number.isFinite(result) ? result : null;
  }
  // Backend validates integer/range; preserve invalid fractional/negative values for a clear status.
  return { center, x: coordinate(x), y: coordinate(y) };
}
