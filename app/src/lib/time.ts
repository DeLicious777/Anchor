/** Formats a millisecond duration as `H:MM:SS` (or `M:SS` under an hour). */
export function formatElapsed(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const pad = (n: number) => n.toString().padStart(2, "0");

  return hours > 0 ? `${hours}:${pad(minutes)}:${pad(seconds)}` : `${minutes}:${pad(seconds)}`;
}

/**
 * Renders a stored UTC instant as the value a `datetime-local` input expects:
 * `YYYY-MM-DDTHH:MM:SS`, in the browser's local zone, with no offset suffix.
 *
 * Built from the local getters rather than slicing `toISOString()`, which reads
 * as the obvious implementation and is wrong — it would print the UTC wall
 * clock into a field the user reads as local time, shifting every correction by
 * the zone offset.
 *
 * Seconds are included and sub-second precision is dropped, because that is the
 * finest an input can express. Callers must therefore not send this value back
 * for a boundary the user did not touch — see `+page.svelte`'s `saveEdit`.
 */
export function toLocalInputValue(instant: string): string {
  const d = new Date(instant);
  const pad = (n: number) => n.toString().padStart(2, "0");

  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}` +
    `T${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
  );
}

/**
 * Inverse of `toLocalInputValue`: a local `datetime-local` value back to a UTC
 * ISO instant. Returns null for an empty or unparseable field so the caller can
 * refuse to send a span it cannot interpret, rather than posting `Invalid Date`
 * and letting the domain reject a value nobody typed.
 */
export function fromLocalInputValue(value: string): string | null {
  if (!value) return null;
  // A date-time string with no offset is parsed as local time, which is exactly
  // what the input means by it.
  const d = new Date(value);

  return Number.isNaN(d.getTime()) ? null : d.toISOString();
}
