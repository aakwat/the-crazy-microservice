import { z } from "zod";

/** The first message for each field: what a form shows under that input. */
export function firstFieldErrors<T>(error: z.ZodError<T>): Partial<Record<keyof T, string>> {
	const { fieldErrors } = z.flattenError(error);
	const first: Partial<Record<keyof T, string>> = {};
	for (const [field, messages] of Object.entries(fieldErrors) as [keyof T, string[] | undefined][]) {
		if (messages?.[0]) first[field] = messages[0];
	}
	return first;
}
