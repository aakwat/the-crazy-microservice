import { z } from "zod";

/** auth's limits (Account.MAX_EMAIL_LENGTH, PasswordCredential password lengths), so mistakes show early. */
export const registerRequestSchema = z.object({
	email: z.email("Please enter a valid email address.").max(254, "Please use a shorter email address."),
	password: z
		.string()
		.min(12, "Please use at least 12 characters.")
		.max(128, "Please use at most 128 characters."),
});

export type RegisterRequest = z.infer<typeof registerRequestSchema>;
