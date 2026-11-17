import type { RegisterRequest } from "../domain/schemas";

/** Sent from the browser, not a server action: the gateway limits sign-ups per visitor's address. */
export async function registerAccount(input: RegisterRequest): Promise<string | undefined> {
	const response = await fetch("/auth/register", {
		method: "POST",
		headers: { "Content-Type": "application/json", Accept: "application/json" },
		body: JSON.stringify(input),
	}).catch(() => null);

	if (response?.ok) return undefined;
	if (response?.status === 409) return "An account with this email already exists. Please sign in.";
	if (response?.status === 429) return "Too many attempts. Please wait a minute and try again.";

	const problem = await response?.json().catch(() => null);
	return problem?.detail ?? "We couldn't create your account. Please try again.";
}
