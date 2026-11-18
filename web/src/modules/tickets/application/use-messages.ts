"use client";

import { useQuery } from "@tanstack/react-query";

import { getMessages } from "../infrastructure/actions";
import { ticketKeys } from "./query-keys";

export function useMessages(id: string) {
	return useQuery({ queryKey: ticketKeys.messages(id), queryFn: () => getMessages(id) });
}
