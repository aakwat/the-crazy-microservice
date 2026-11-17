import "server-only";

import { cache } from "react";

import { gatewayFetch } from "@/shared/lib/api/gateway-fetch.server";
import { ProblemError } from "@/shared/lib/api/problem";
import { API_URL } from "@/shared/lib/api/safe-fetch.server";

export type SessionUser = { sub: string; email: string };

/** Who is signed in, asked of the gateway once per request; null when nobody is. */
export const getSessionUser = cache(async (): Promise<SessionUser | null> => {
	try {
		const response = await gatewayFetch(`${API_URL}/bff/user`);
		return (await response.json()) as SessionUser;
	} catch (err) {
		if (err instanceof ProblemError && err.problem.status === 401) return null;
		throw err;
	}
});
