// Create tenant record using Minerva CLI
// Replaces direct SQL: INSERT INTO tenants (...)

export async function main(family_name: string) {
  const cmd = new Deno.Command("minerva", {
    args: ["onboarding", "create-tenant", "--name", family_name],
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    throw new Error(`minerva failed: ${error}`);
  }

  const result = JSON.parse(new TextDecoder().decode(stdout));
  
  return { 
    tenant_id: result.tenant_id, 
    tenant_name: result.name, 
    created_at: result.created_at 
  };
}
