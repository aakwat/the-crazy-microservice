"use server";

import { apiClient } from "@/shared/lib/api/client.server";
import { upstreamUnavailable } from "@/shared/lib/api/problem";
import { safeAction } from "@/shared/lib/safe-action";

import { createTicketRequestSchema, ticketIdSchema } from "../domain/schemas";
import type { CreateTicketRequest, Ticket, TicketPage } from "../domain/types";

export async function getMyTickets(): Promise<TicketPage> {
	const { data } = await apiClient.GET("/api/tickets");

	if (!data) throw upstreamUnavailable("Tickets returned no data");

	return data;
}

export async function getTicket(id: string): Promise<Ticket> {
	const { data } = await apiClient.GET("/api/tickets/{id}", {
		params: { path: { id: ticketIdSchema.parse(id) } },
	});

	if (!data) throw upstreamUnavailable("Ticket returned no data");

	return data;
}

export const createTicketAction = safeAction(async (input: CreateTicketRequest): Promise<Ticket> => {
	const { data } = await apiClient.POST("/api/tickets", { body: createTicketRequestSchema.parse(input) });

	if (!data) throw upstreamUnavailable("Create ticket returned no data");

	return data;
});
