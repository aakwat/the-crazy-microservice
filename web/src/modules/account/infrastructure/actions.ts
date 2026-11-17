"use server";

import { getSessionUser, type SessionUser } from "@/shared/lib/auth/session.server";

export async function getCurrentUser(): Promise<SessionUser | null> {
	return getSessionUser();
}
