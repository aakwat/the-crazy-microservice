import { RegisterForm } from "@/modules/account";

export default function Page() {
	return (
		<section className="grid gap-6">
			<h1 className="text-2xl font-semibold">Create your account</h1>
			<RegisterForm />
		</section>
	);
}
