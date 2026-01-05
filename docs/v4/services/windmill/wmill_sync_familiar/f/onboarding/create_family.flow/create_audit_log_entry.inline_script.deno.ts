// Create audit log entry using Minerva CLI
// Replaces direct SQL: INSERT INTO audit_log (...)

export async function main(user_id: string, user_email: string, tenant_id: string, tenant_name: string) {
  const metadataJson = JSON.stringify({ tenant_name });
  
  const cmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "audit-log",
      "--action", "tenant_created",
      "--user-id", user_id,
      "--user-email", user_email,
      "--resource-type", "tenant",
      "--resource-id", tenant_id,
      "--metadata-json", metadataJson,
    ],
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    console.warn(`Warning: Audit log failed: ${error}`);
    // Don't throw - audit logging failure shouldn't block family creation
  }

  return { logged: code === 0 };
}
