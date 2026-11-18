import { describe, expect, it } from "vitest";

import { canConfirm, canReply, canWithdraw } from "./lifecycle";
import type { TicketStatus } from "./types";

const ALL: TicketStatus[] = ["new", "open", "pending", "resolved", "closed"];

function allowed(rule: (status: TicketStatus) => boolean): TicketStatus[] {
	return ALL.filter(rule);
}

describe("customer actions", () => {
	it("replies until the ticket is closed", () => {
		expect(allowed(canReply)).toEqual(["new", "open", "pending", "resolved"]);
	});

	it("confirms only a resolved ticket", () => {
		expect(allowed(canConfirm)).toEqual(["resolved"]);
	});

	it("withdraws anything not yet resolved", () => {
		expect(allowed(canWithdraw)).toEqual(["new", "open", "pending"]);
	});
});
