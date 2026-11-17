export function resolveRequestId(inbound: string | null | undefined): string {
	return inbound?.trim() || crypto.randomUUID();
}
