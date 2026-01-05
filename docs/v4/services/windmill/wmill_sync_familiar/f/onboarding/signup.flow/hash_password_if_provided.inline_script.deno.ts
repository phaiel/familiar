export async function main(password: string | null, has_password: boolean) {
  if (!has_password || !password) return { password_hash: null };
  const encoder = new TextEncoder();
  const salt = 'familiar_salt_v1';
  const data = encoder.encode(password + salt);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  return { password_hash: hashHex };
}