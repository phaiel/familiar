// Add user as admin member using Minerva CLI
// Replaces direct SQL: INSERT INTO tenant_members (...)

export async function main(tenant_id: string, user_id: string, user_name: string, user_email: string) {
  const cmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "add-member",
      "--tenant-id", tenant_id,
      "--user-id", user_id,
      "--role", "admin",
      "--name", user_name,
      "--email", user_email,
    ],
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    throw new Error(`minerva failed: ${error}`);
  }

  return { added: true, role: 'admin' };
}
