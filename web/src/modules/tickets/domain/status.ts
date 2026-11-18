import type { CloseReason, TicketStatus } from "./types";

/** What customers read for each status: plain words, no internal terms (docs/business-rules.md, 13.1). */
const LABELS: Record<TicketStatus, string> = {
	new: "Received",
	open: "In progress",
	pending: "Waiting for your reply",
	resolved: "Solved, please confirm",
	closed: "Closed",
};

const CLOSE_REASONS: Record<NonNullable<CloseReason>, string> = {
	confirmed: "You confirmed it is solved.",
	withdrawn: "You withdrew this ticket.",
};

export function statusLabel(status: string): string {
	return LABELS[status as TicketStatus] ?? "Unknown";
}

export function closeReasonLabel(reason: CloseReason): string | undefined {
	return reason ? CLOSE_REASONS[reason] : undefined;
}
