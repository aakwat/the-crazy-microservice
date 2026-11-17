import { z } from "zod";

/** The limits the tickets service enforces, so mistakes show before anything is sent. */
export const MAX_TITLE_CHARS = 200;
export const MAX_BODY_CHARS = 10_000;

export const createTicketRequestSchema = z.object({
	title: z
		.string()
		.trim()
		.min(1, "Please add a title.")
		.max(MAX_TITLE_CHARS, `Please keep the title under ${MAX_TITLE_CHARS} characters.`),
	body: z
		.string()
		.trim()
		.min(1, "Please describe the problem.")
		.max(MAX_BODY_CHARS, `Please keep the description under ${MAX_BODY_CHARS} characters.`),
});

export const ticketIdSchema = z.uuid();
