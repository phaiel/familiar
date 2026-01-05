// Create audit log entry using Minerva CLI
// Replaces direct SQL: INSERT INTO audit_log (...)

export async function main(
  user_id: string, 
  email: string, 
  ip_address: string | null, 
  user_agent: string | null, 
  joined_family_id: string | null
) {
  const metadataJson = JSON.stringify({ joined_family_id });
  
  const args = [
    "onboarding", "audit-log",
    "--action", "signup",
    "--user-id", user_id,
    "--user-email", email,
    "--resource-type", "user",
    "--resource-id", user_id,
    "--metadata-json", metadataJson,
  ];

  const cmd = new Deno.Command("minerva", {
    args,
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    console.warn(`Warning: Audit log failed: ${error}`);
    // Don't throw - audit logging failure shouldn't block signup
  }

  return { logged: code === 0 };
}
