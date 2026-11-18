"use client";

import { Alert, AlertDescription, AlertTitle } from "@/shared/components/ui/alert";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/shared/components/ui/card";
import { formatExact } from "@/shared/lib/format";

import { useTicket } from "../application/use-ticket";
import { canReply } from "../domain/lifecycle";
import { closeReasonLabel } from "../domain/status";
import { MessageThread } from "./message-thread";
import { ReplyForm } from "./reply-form";
import { StatusBadge } from "./status-badge";
import { TicketActions } from "./ticket-actions";

export interface TicketDetailProps {
	id: string;
}

export function TicketDetail(props: TicketDetailProps) {
	const { id } = props;
	const ticket = useTicket(id);

	if (ticket.isPending) return <p className="text-sm text-muted-foreground">Loading…</p>;

	if (ticket.isError) {
		return (
			<Alert variant="destructive">
				<AlertTitle>We couldn’t find that ticket.</AlertTitle>
				<AlertDescription>Check the link, or go back to your tickets.</AlertDescription>
			</Alert>
		);
	}

	const { status, close_reason: closeReason } = ticket.data;
	const closedBecause = closeReasonLabel(closeReason);

	return (
		<div className="grid max-w-2xl gap-6">
			<Card>
				<CardHeader>
					<CardDescription className="font-mono">{ticket.data.reference}</CardDescription>
					<CardTitle className="flex items-center justify-between gap-3">
						{ticket.data.title}
						<StatusBadge status={status} />
					</CardTitle>
					<CardDescription>Sent {formatExact(ticket.data.created_at, "en")}</CardDescription>
				</CardHeader>
				<CardContent className="grid gap-3">
					<p className="whitespace-pre-wrap">{ticket.data.body}</p>
					{closedBecause && <p className="text-sm text-muted-foreground">{closedBecause}</p>}
				</CardContent>
			</Card>

			<section aria-labelledby="conversation-heading" className="grid gap-4">
				<h2 id="conversation-heading" className="text-lg font-medium">
					Conversation
				</h2>
				<MessageThread id={id} />
				{canReply(status) && <ReplyForm id={id} status={status} />}
				<TicketActions id={id} status={status} />
			</section>
		</div>
	);
}
