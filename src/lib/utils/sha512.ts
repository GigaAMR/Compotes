export default async function sha512(str): Promise<string> {
	let encodedText = new TextEncoder().encode(str);
	let buffer = await crypto.subtle.digest('SHA-512', encodedText);

	return Array.prototype.map
		.call(new Uint8Array(buffer), function (x: number): string {
			return ('00' + x.toString(16)).slice(-2);
		})
		.join('');
}
