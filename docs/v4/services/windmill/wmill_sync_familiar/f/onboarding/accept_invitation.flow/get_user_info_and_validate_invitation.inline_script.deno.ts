// Get user info and validate invitation using Minerva CLI
// Replaces direct SQL: SELECT from users, family_invitations, tenants, tenant_members

export async function main(user_id: string, invite_code: string) {
  // Validate invitation for this user
  const cmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "validate-invitation",
      "--user-id", user_id,
      "--invite-code", invite_code,
    ],
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    throw new Error(`minerva failed: ${error}`);
  }

  const validation = JSON.parse(new TextDecoder().decode(stdout));
  
  if (!validation.valid) {
    // Map error messages to expected format
    const errorMsg = validation.error || 'Invalid invitation';
    let errorCode = 'INVALID_CODE';
    
    if (errorMsg.includes('expired')) errorCode = 'EXPIRED';
    else if (errorMsg.includes('exhausted') || errorMsg.includes('limit')) errorCode = 'LIMIT_REACHED';
    else if (errorMsg.includes('already a member')) errorCode = 'ALREADY_MEMBER';
    
    throw new Error(JSON.stringify({ code: errorCode, message: errorMsg }));
  }

  return {
    user_id,
    invitation_id: validation.invitation_id,
    tenant_id: validation.tenant_id,
    tenant_name: validation.tenant_name,
    role: validation.role || 'member'
  };
}
