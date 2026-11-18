import { describe, expect, it } from "vitest";

import { closeReasonLabel, statusLabel } from "./status";

describe("statusLabel", () => {
	it("shows customers plain words for every status", () => {
		expect(statusLabel("new")).toBe("Received");
		expect(statusLabel("open")).toBe("In progress");
		expect(statusLabel("pending")).toBe("Waiting for your reply");
		expect(statusLabel("resolved")).toBe("Solved, please confirm");
		expect(statusLabel("closed")).toBe("Closed");
	});

	it("never shows a raw internal value", () => {
		expect(statusLabel("escalated")).toBe("Unknown");
	});
});

describe("closeReasonLabel", () => {
	it("explains why a ticket is closed, and says nothing while it is open", () => {
		expect(closeReasonLabel("confirmed")).toBe("You confirmed it is solved.");
		expect(closeReasonLabel("withdrawn")).toBe("You withdrew this ticket.");
		expect(closeReasonLabel(null)).toBeUndefined();
	});
});
