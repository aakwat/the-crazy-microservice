import { NewTicketForm } from "@/modules/tickets";

export default function Page() {
	return (
		<section className="grid gap-6">
			<h1 className="text-2xl font-semibold">Tell us about the problem</h1>
			<NewTicketForm />
		</section>
	);
}
