"use client";

import { useQuery } from "@tanstack/react-query";

import { getTicket } from "../infrastructure/actions";
import { ticketKeys } from "./query-keys";

export function useTicket(id: string) {
	return useQuery({ queryKey: ticketKeys.detail(id), queryFn: () => getTicket(id) });
}
