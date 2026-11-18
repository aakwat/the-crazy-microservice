import type { Message } from "./types";

/** Staff reply as the company, never as a named person (docs/business-rules.md, COMM-003). */
export function senderLabel(from: Message["from"]): string {
	return from === "support" ? "Support" : "You";
}
