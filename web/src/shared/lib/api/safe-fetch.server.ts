import "server-only";

import { headers } from "next/headers";

import { env } from "@/shared/config/env";

import { problemType } from "./error-code";
import { isProblem, problemSchema, ProblemError } from "./problem";
import { resolveRequestId } from "./request-id";

export const API_URL = env.API_URL;

/** Matches the deadline Next's own image optimizer uses for upstream fetches. */
const UPSTREAM_TIMEOUT_MS = 7000;

// Next exposes no socket peer, so the only X-Forwarded-For entry we can trust is
// the one our own edge appended: count back from the right, one per trusted hop.
export function clientIpFromForwardedFor(xff: string | null, hops: number): string | undefined {
	if (!Number.isInteger(hops) || hops < 1) return undefined;

	return (xff ?? "")
		.split(",")
		.map((entry) => entry.trim())
		.filter(Boolean)
		.at(-hops);
}

export async function safeFetch(
	input: string | URL | Request,
	init?: RequestInit,
): Promise<Response> {
	const incoming = await headers();
	const requestId = resolveRequestId(incoming.get("x-request-id"));

	const reqHeaders = new Headers(input instanceof Request ? input.headers : undefined);
	if (init?.headers) {
		new Headers(init.headers).forEach((value, key) => reqHeaders.set(key, value));
	}
	reqHeaders.set("x-request-id", requestId);

	const clientIp = clientIpFromForwardedFor(incoming.get("x-forwarded-for"), env.TRUSTED_PROXY_HOPS);
	if (clientIp) reqHeaders.set("x-forwarded-for", clientIp);
	else reqHeaders.delete("x-forwarded-for");

	if (!reqHeaders.has("user-agent")) {
		const ua = incoming.get("user-agent");
		if (ua) reqHeaders.set("user-agent", ua);
	}

	let res: Response;

	try {
		// A tarpitting upstream otherwise holds the request open, and on the
		// storefront anyone can open those without signing in (CWE-400).
		res = await fetch(input, { signal: AbortSignal.timeout(UPSTREAM_TIMEOUT_MS), ...init, headers: reqHeaders });
	} catch {
		throw new ProblemError(
			problemSchema.parse({
				type: problemType("upstream-unavailable"),
				title: "Our servers aren't responding right now.",
				status: 503,
			}),
		);
	}

	if (res.ok) return res;

	const body = await res.clone().json().catch(() => null);

	if (isProblem(body)) throw new ProblemError(body);

	throw new ProblemError(
		problemSchema.parse({
			type: "about:blank",
			title: `Upstream ${res.status}`,
			status: res.status,
		}),
	);
}
