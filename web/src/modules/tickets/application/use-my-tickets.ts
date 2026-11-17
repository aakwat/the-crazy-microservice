"use client";

import { useQuery } from "@tanstack/react-query";

import { getMyTickets } from "../infrastructure/actions";
import { ticketKeys } from "./query-keys";

export function useMyTickets() {
	return useQuery({ queryKey: ticketKeys.all, queryFn: () => getMyTickets() });
}
