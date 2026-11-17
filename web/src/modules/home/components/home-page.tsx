"use client";

import { useCurrentUser } from "@/modules/account";
import { TicketList } from "@/modules/tickets";
import { buttonVariants } from "@/shared/components/ui/button";

export function HomePage() {
	const user = useCurrentUser();

	if (user.isPending) return null;

	if (!user.data) {
		return (
			<section className="grid max-w-xl justify-items-start gap-4">
				<h1 className="text-2xl font-semibold">How can we help?</h1>
				<p className="text-muted-foreground">
					Sign in to tell us about a problem and follow it until it is solved.
				</p>
				<a href="/bff/login" className={buttonVariants()}>
					Sign in
				</a>
			</section>
		);
	}

	return (
		<section className="grid gap-6">
			<h1 className="text-2xl font-semibold">Your tickets</h1>
			<TicketList />
		</section>
	);
}
