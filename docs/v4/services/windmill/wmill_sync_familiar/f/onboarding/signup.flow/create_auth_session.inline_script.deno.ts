// Create auth session using Minerva CLI
// Replaces direct SQL: INSERT INTO auth_sessions (user_id, token_hash, ...)

export async function main(user_id: string, ip_address: string | null, user_agent: string | null) {
  // Note: ip_address and user_agent are not currently passed to minerva CLI
  // They can be added later if needed for audit purposes
  
  const cmd = new Deno.Command("minerva", {
    args: ["onboarding", "create-session", "--user-id", user_id],
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    throw new Error(`minerva failed: ${error}`);
  }

  return JSON.parse(new TextDecoder().decode(stdout));
}
