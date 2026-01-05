// Update user's primary tenant using Minerva CLI
// Replaces direct SQL: UPDATE users SET primary_tenant_id = $1 ...

export async function main(user_id: string, tenant_id: string) {
  const cmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "set-primary-tenant",
      "--user-id", user_id,
      "--tenant-id", tenant_id,
    ],
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    throw new Error(`minerva failed: ${error}`);
  }

  return { updated: true };
}
