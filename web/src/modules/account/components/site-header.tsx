"use client";

import Link from "next/link";

import { Button, buttonVariants } from "@/shared/components/ui/button";

import { useCurrentUser } from "../application/use-current-user";

export function SiteHeader() {
	const user = useCurrentUser();

	return (
		<header className="border-b">
			<nav className="mx-auto flex max-w-4xl items-center justify-between gap-4 px-6 py-4">
				<Link href="/" className="font-semibold">
					Support
				</Link>

				{user.data && (
					<div className="flex items-center gap-3">
						<span className="text-sm text-muted-foreground">{user.data.email}</span>
						<Link href="/tickets/new" className={buttonVariants({ size: "sm" })}>
							New ticket
						</Link>
						{/* A plain form: the gateway ends the session and sends the browser back here. */}
						<form method="post" action="/bff/logout">
							<Button type="submit" variant="outline" size="sm">
								Sign out
							</Button>
						</form>
					</div>
				)}

				{user.data === null && (
					<div className="flex items-center gap-3">
						<Link href="/register" className={buttonVariants({ variant: "ghost", size: "sm" })}>
							Create account
						</Link>
						{/* A full page load: sign-in belongs to the gateway, not to this app. */}
						<a href="/bff/login" className={buttonVariants({ size: "sm" })}>
							Sign in
						</a>
					</div>
				)}
			</nav>
		</header>
	);
}
