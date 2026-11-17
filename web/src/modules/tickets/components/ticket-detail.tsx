"use client";

import { Alert, AlertDescription, AlertTitle } from "@/shared/components/ui/alert";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/shared/components/ui/card";
import { formatExact } from "@/shared/lib/format";

import { useTicket } from "../application/use-ticket";
import { StatusBadge } from "./status-badge";

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

	return (
		<Card className="max-w-2xl">
			<CardHeader>
				<CardDescription className="font-mono">{ticket.data.reference}</CardDescription>
				<CardTitle className="flex items-center justify-between gap-3">
					{ticket.data.title}
					<StatusBadge status={ticket.data.status} />
				</CardTitle>
				<CardDescription>Sent {formatExact(ticket.data.created_at, "en")}</CardDescription>
			</CardHeader>
			<CardContent>
				<p className="whitespace-pre-wrap">{ticket.data.body}</p>
			</CardContent>
		</Card>
	);
}
