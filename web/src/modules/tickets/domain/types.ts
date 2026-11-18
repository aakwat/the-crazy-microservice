import type { components } from "@/shared/lib/api/contracts/tickets";

export type Ticket = components["schemas"]["Ticket"];
export type TicketSummary = components["schemas"]["TicketSummary"];
export type TicketPage = components["schemas"]["TicketPage"];
export type TicketStatus = components["schemas"]["Status"];
export type CloseReason = components["schemas"]["CloseReason"];
export type CreateTicketRequest = components["schemas"]["CreateTicket"];
export type Message = components["schemas"]["Message"];
export type ReplyRequest = Pick<components["schemas"]["PostMessage"], "body">;
