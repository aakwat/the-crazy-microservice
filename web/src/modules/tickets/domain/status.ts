/** What customers read for each status: plain words, no internal terms (docs/business-rules.md, 13.1). */
const LABELS: Record<string, string> = {
	open: "Received",
	in_progress: "In progress",
	resolved: "Resolved",
	closed: "Closed",
};

export function statusLabel(status: string): string {
	return LABELS[status] ?? "Unknown";
}
