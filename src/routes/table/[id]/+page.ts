export const prerender = false;
export const ssr = true;

export async function load({ params }) {
	return {
		id: params.id
	};
}
