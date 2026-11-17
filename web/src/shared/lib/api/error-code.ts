/** The codes this app originates; backend codes pass through as plain strings. One language, so no catalog. */
export type ErrorCode = "validation-error" | "upstream-unavailable" | "unknown";

/** Constrains the literal without narrowing the field, which also carries backend codes. */
export function frontendCode(code: ErrorCode): string {
	return code;
}

/** Builds an RFC 9457 `type` for a code this app originates rather than relays. */
export function problemType(code: ErrorCode): string {
	return `/problems/${code}`;
}

export function slugFromProblemType(type: string | undefined): string {
	if (!type) return frontendCode("unknown");
	const match = type.match(/\/problems\/([^/?#]+)/);
	return match?.[1] ?? frontendCode("unknown");
}
