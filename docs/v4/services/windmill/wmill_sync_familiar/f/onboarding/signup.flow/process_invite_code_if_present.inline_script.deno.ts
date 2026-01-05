// Process invite code if present using Minerva CLI
// Replaces direct SQL: SELECT from family_invitations, INSERT INTO tenant_members, UPDATE users, UPDATE family_invitations

export async function main(
  user_id: string, 
  user_name: string, 
  user_email: string, 
  invite_code: string | null, 
  has_invite_code: boolean
) {
  // If no invite code, user needs to create their own family
  if (!has_invite_code || !invite_code) {
    return { needs_family: true, joined_family_id: null, joined_family_name: null };
  }

  // Validate the invitation for this user
  const validateCmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "validate-invitation",
      "--user-id", user_id,
      "--invite-code", invite_code,
    ],
    stdout: "piped",
    stderr: "piped",
  });

  const { code: validateCode, stdout: validateStdout, stderr: validateStderr } = await validateCmd.output();

  if (validateCode !== 0) {
    const error = new TextDecoder().decode(validateStderr);
    return { needs_family: true, joined_family_id: null, invite_error: `Validation failed: ${error}` };
  }

  const validation = JSON.parse(new TextDecoder().decode(validateStdout));
  
  if (!validation.valid) {
    return { needs_family: true, joined_family_id: null, invite_error: validation.error || 'Invalid invite code' };
  }

  // Add user as member to the tenant
  const addMemberCmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "add-member",
      "--tenant-id", validation.tenant_id,
      "--user-id", user_id,
      "--role", validation.role || "member",
      "--name", user_name,
      "--email", user_email,
    ],
    stdout: "piped",
    stderr: "piped",
  });

  const { code: addCode, stderr: addStderr } = await addMemberCmd.output();

  if (addCode !== 0) {
    const error = new TextDecoder().decode(addStderr);
    return { needs_family: true, joined_family_id: null, invite_error: `Failed to add member: ${error}` };
  }

  // Set user's primary tenant
  const setPrimaryCmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "set-primary-tenant",
      "--user-id", user_id,
      "--tenant-id", validation.tenant_id,
    ],
    stdout: "piped",
    stderr: "piped",
  });

  const { code: setCode, stderr: setStderr } = await setPrimaryCmd.output();

  if (setCode !== 0) {
    const error = new TextDecoder().decode(setStderr);
    console.warn(`Warning: Failed to set primary tenant: ${error}`);
  }

  // Increment invitation usage
  const incrementCmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "increment-invite",
      "--invitation-id", validation.invitation_id,
    ],
    stdout: "piped",
    stderr: "piped",
  });

  const { code: incCode, stderr: incStderr } = await incrementCmd.output();

  if (incCode !== 0) {
    const error = new TextDecoder().decode(incStderr);
    console.warn(`Warning: Failed to increment invitation usage: ${error}`);
  }

  return { 
    needs_family: false, 
    joined_family_id: validation.tenant_id, 
    joined_family_name: validation.tenant_name,
    role: validation.role 
  };
}
