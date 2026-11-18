"use server";

import { apiClient } from "@/shared/lib/api/client.server";
import { requireData } from "@/shared/lib/api/require-data";
import { safeAction } from "@/shared/lib/safe-action";

import { createTicketRequestSchema, replyRequestSchema, ticketIdSchema } from "../domain/schemas";
import type { CreateTicketRequest, Message, ReplyRequest, Ticket, TicketPage } from "../domain/types";

function ticketPath(id: string) {
	return { params: { path: { id: ticketIdSchema.parse(id) } } };
}

export async function getMyTickets(): Promise<TicketPage> {
	const { data } = await apiClient.GET("/api/tickets");

	return requireData(data, "Tickets");
}

export async function getTicket(id: string): Promise<Ticket> {
	const { data } = await apiClient.GET("/api/tickets/{id}", ticketPath(id));

	return requireData(data, "Ticket");
}

export async function getMessages(id: string): Promise<Message[]> {
	const { data } = await apiClient.GET("/api/tickets/{id}/messages", ticketPath(id));

	return requireData(data, "Messages").items;
}

export const createTicketAction = safeAction(async (input: CreateTicketRequest): Promise<Ticket> => {
	const { data } = await apiClient.POST("/api/tickets", { body: createTicketRequestSchema.parse(input) });

	return requireData(data, "Create ticket");
});

/** A customer sends only text; notes and status moves are for staff. */
export const replyAction = safeAction(async (input: { id: string } & ReplyRequest): Promise<Message> => {
	const { data } = await apiClient.POST("/api/tickets/{id}/messages", {
		...ticketPath(input.id),
		body: { ...replyRequestSchema.parse({ body: input.body }), internal: false },
	});

	return requireData(data, "Reply");
});

export const confirmTicketAction = safeAction(async (id: string): Promise<Ticket> => {
	const { data } = await apiClient.POST("/api/tickets/{id}/confirm", ticketPath(id));

	return requireData(data, "Confirm");
});

export const withdrawTicketAction = safeAction(async (id: string): Promise<Ticket> => {
	const { data } = await apiClient.POST("/api/tickets/{id}/withdraw", ticketPath(id));

	return requireData(data, "Withdraw");
});
