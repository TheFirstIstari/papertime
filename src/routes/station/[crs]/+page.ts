export const csr = true;

export async function load({ params }) {
	return {
		crs: params.crs
	};
}
