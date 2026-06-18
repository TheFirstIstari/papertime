export const prerender = false;
export const ssr = true;

export async function load({ params, url }) {
	return {
		id: params.id,
		from: url.searchParams.get('from') ?? '',
		to: url.searchParams.get('to') ?? ''
	};
}
