// Validate user exists and needs a family using Minerva CLI
// Replaces direct SQL: SELECT from users, SELECT from tenant_members

export async function main(user_id: string, family_name: string) {
  // Validate family name locally (no need for database call)
  if (!family_name || family_name.trim().length < 2) {
    throw new Error(JSON.stringify({ 
      code: 'INVALID_NAME', 
      message: 'Family name must be at least 2 characters' 
    }));
  }

  // Check if user needs a family
  const cmd = new Deno.Command("minerva", {
    args: ["onboarding", "check-needs-family", "--user-id", user_id],
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    try {
      const parsed = JSON.parse(error);
      if (parsed.code === "USER_NOT_FOUND") {
        throw new Error(JSON.stringify({ code: 'USER_NOT_FOUND', message: 'User not found' }));
      }
    } catch {}
    throw new Error(`minerva failed: ${error}`);
  }

  const result = JSON.parse(new TextDecoder().decode(stdout));
  
  if (!result.needs_family) {
    throw new Error(JSON.stringify({ 
      code: 'ALREADY_HAS_FAMILY', 
      message: 'User already belongs to a family' 
    }));
  }

  return { 
    valid: true, 
    user_id,
    family_name: family_name.trim()
  };
}
