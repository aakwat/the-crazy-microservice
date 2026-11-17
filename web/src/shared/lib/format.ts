// Exact moments for records (audit trails); the activity panels use the
// relative form below.
export function formatExact(iso: string, locale: string): string {
	return new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeStyle: "medium" }).format(new Date(iso));
}

export function formatRelative(iso: string, locale: string): string {
	const diffSecs = Math.round((new Date(iso).getTime() - Date.now()) / 1000);
	const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
	const abs = Math.abs(diffSecs);
	if (abs < 60) return rtf.format(diffSecs, "second");
	if (abs < 3600) return rtf.format(Math.round(diffSecs / 60), "minute");
	if (abs < 86_400) return rtf.format(Math.round(diffSecs / 3600), "hour");
	return rtf.format(Math.round(diffSecs / 86_400), "day");
}
