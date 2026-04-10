/**
 * Check if a date falls within the current week (Monday–Sunday).
 */
export function isThisWeek(dateTime: string, now: Date = new Date()): boolean {
  const date = new Date(dateTime);

  // Monday 00:00:00 of the current week
  const monday = new Date(now);
  const day = monday.getDay();
  const diff = day === 0 ? 6 : day - 1; // Sunday = 6 days back, else day - 1
  monday.setDate(monday.getDate() - diff);
  monday.setHours(0, 0, 0, 0);

  // Sunday 23:59:59.999 of the current week
  const sunday = new Date(monday);
  sunday.setDate(monday.getDate() + 6);
  sunday.setHours(23, 59, 59, 999);

  return date >= monday && date <= sunday;
}

/**
 * Check if a date is today.
 */
export function isToday(dateTime: string, now: Date = new Date()): boolean {
  const date = new Date(dateTime);
  return (
    date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate()
  );
}
