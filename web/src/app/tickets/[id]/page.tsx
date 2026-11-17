import { TicketDetail } from "@/modules/tickets";

export default async function Page(props: PageProps<"/tickets/[id]">) {
	const { id } = await props.params;

	return <TicketDetail id={id} />;
}
