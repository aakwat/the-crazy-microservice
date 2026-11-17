"use client";

import { useQuery } from "@tanstack/react-query";

import { getCurrentUser } from "../infrastructure/actions";

/** `data` is the signed-in person, `null` when nobody is, `undefined` while still asking. */
export function useCurrentUser() {
	return useQuery({ queryKey: ["current-user"], queryFn: () => getCurrentUser() });
}
