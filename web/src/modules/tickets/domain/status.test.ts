import { describe, expect, it } from "vitest";

import { statusLabel } from "./status";

describe("statusLabel", () => {
	it("shows customers plain words for every status", () => {
		expect(statusLabel("open")).toBe("Received");
		expect(statusLabel("in_progress")).toBe("In progress");
		expect(statusLabel("resolved")).toBe("Resolved");
		expect(statusLabel("closed")).toBe("Closed");
	});

	it("never shows a raw internal value", () => {
		expect(statusLabel("escalated")).toBe("Unknown");
	});
});
