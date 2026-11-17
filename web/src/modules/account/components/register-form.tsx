"use client";

import { Alert, AlertDescription, AlertTitle } from "@/shared/components/ui/alert";
import { Button, buttonVariants } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";

import { useRegister } from "../application/use-register";

export function RegisterForm() {
	const { onSubmit, fieldErrors, failure, pending, created } = useRegister();

	if (created) {
		return (
			<Alert className="max-w-md">
				<AlertTitle>Your account is ready.</AlertTitle>
				<AlertDescription className="grid justify-items-start gap-3">
					Sign in to send us your first ticket.
					<a href="/bff/login" className={buttonVariants({ size: "sm" })}>
						Sign in
					</a>
				</AlertDescription>
			</Alert>
		);
	}

	return (
		<form onSubmit={onSubmit} className="grid max-w-md gap-5" noValidate>
			<div className="grid gap-2">
				<Label htmlFor="email">Email</Label>
				<Input id="email" name="email" type="email" autoComplete="email" aria-invalid={!!fieldErrors.email} />
				{fieldErrors.email && <p className="text-sm text-destructive">{fieldErrors.email}</p>}
			</div>

			<div className="grid gap-2">
				<Label htmlFor="password">Password</Label>
				<Input
					id="password"
					name="password"
					type="password"
					autoComplete="new-password"
					aria-invalid={!!fieldErrors.password}
				/>
				{fieldErrors.password && <p className="text-sm text-destructive">{fieldErrors.password}</p>}
			</div>

			{failure && (
				<Alert variant="destructive">
					<AlertDescription>{failure}</AlertDescription>
				</Alert>
			)}

			<Button type="submit" disabled={pending} className="justify-self-start">
				{pending ? "Creating…" : "Create account"}
			</Button>
		</form>
	);
}
