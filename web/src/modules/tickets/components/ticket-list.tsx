"use client";

import Link from "next/link";

import { Alert, AlertDescription, AlertTitle } from "@/shared/components/ui/alert";
import { buttonVariants } from "@/shared/components/ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "@/shared/components/ui/card";
import { formatExact } from "@/shared/lib/format";

import { useMyTickets } from "../application/use-my-tickets";
import { StatusBadge } from "./status-badge";

export function TicketList() {
	const tickets = useMyTickets();

	if (tickets.isPending) return <p className="text-sm text-muted-foreground">Loading your tickets…</p>;

	if (tickets.isError) {
		return (
			<Alert variant="destructive">
				<AlertTitle>We couldn’t load your tickets.</AlertTitle>
				<AlertDescription>
					If you were away for a while, <a href="/bff/login">sign in</a> again.
				</AlertDescription>
			</Alert>
		);
	}

	if (tickets.data.items.length === 0) {
		return (
			<div className="grid justify-items-start gap-3">
				<p className="text-sm text-muted-foreground">You have no tickets yet.</p>
				<Link href="/tickets/new" className={buttonVariants()}>
					Ask for help
				</Link>
			</div>
		);
	}

	return (
		<ul className="grid gap-3">
			{tickets.data.items.map((ticket) => (
				<li key={ticket.id}>
					<Link href={`/tickets/${ticket.id}`} className="block rounded-xl focus-visible:outline-2">
						<Card className="transition-colors hover:bg-muted/50">
							<CardHeader>
								<CardTitle className="flex items-center justify-between gap-3">
									<span>
										<span className="mr-2 font-mono text-muted-foreground">{ticket.reference}</span>
										{ticket.title}
									</span>
									<StatusBadge status={ticket.status} />
								</CardTitle>
								<CardDescription>Updated {formatExact(ticket.updated_at, "en")}</CardDescription>
							</CardHeader>
						</Card>
					</Link>
				</li>
			))}
		</ul>
	);
}
