// Add user as family member using Minerva CLI
// Replaces direct SQL: SELECT from tenant_members, INSERT INTO tenant_members, UPDATE users

export async function main(tenant_id: string, user_id: string, user_name: string, user_email: string, role: string) {
  // Check if user needs a family (to determine is_primary)
  const checkCmd = new Deno.Command("minerva", {
    args: ["onboarding", "check-needs-family", "--user-id", user_id],
    stdout: "piped",
    stderr: "piped",
  });

  const { code: checkCode, stdout: checkStdout } = await checkCmd.output();
  
  let is_primary = false;
  if (checkCode === 0) {
    const checkResult = JSON.parse(new TextDecoder().decode(checkStdout));
    is_primary = checkResult.needs_family;
  }

  // Add user as member
  const addCmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "add-member",
      "--tenant-id", tenant_id,
      "--user-id", user_id,
      "--role", role,
      "--name", user_name,
      "--email", user_email,
    ],
    stdout: "piped",
    stderr: "piped",
  });

  const { code: addCode, stderr: addStderr } = await addCmd.output();

  if (addCode !== 0) {
    const error = new TextDecoder().decode(addStderr);
    throw new Error(`minerva failed: ${error}`);
  }

  // Set primary tenant if this is user's first family
  if (is_primary) {
    const setPrimaryCmd = new Deno.Command("minerva", {
      args: [
        "onboarding", "set-primary-tenant",
        "--user-id", user_id,
        "--tenant-id", tenant_id,
      ],
      stdout: "piped",
      stderr: "piped",
    });

    const { code: setCode, stderr: setStderr } = await setPrimaryCmd.output();

    if (setCode !== 0) {
      const error = new TextDecoder().decode(setStderr);
      console.warn(`Warning: Failed to set primary tenant: ${error}`);
    }
  }

  return { added: true, is_primary };
}
