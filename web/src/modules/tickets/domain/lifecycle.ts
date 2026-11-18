import type { TicketStatus } from "./types";

/** What a customer may do in each status; tickets enforces the same rules (docs/business-rules.md, 13.2). */
export function canReply(status: TicketStatus): boolean {
	return status !== "closed";
}

export function canConfirm(status: TicketStatus): boolean {
	return status === "resolved";
}

/** A resolved ticket is confirmed or reopened by replying, never withdrawn. */
export function canWithdraw(status: TicketStatus): boolean {
	return status === "new" || status === "open" || status === "pending";
}
