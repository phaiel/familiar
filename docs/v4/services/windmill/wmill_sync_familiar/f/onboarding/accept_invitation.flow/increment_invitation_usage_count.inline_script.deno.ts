// Increment invitation usage count using Minerva CLI
// Replaces direct SQL: UPDATE family_invitations SET use_count = use_count + 1 ...

export async function main(invitation_id: string) {
  const cmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "increment-invite",
      "--invitation-id", invitation_id,
    ],
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    throw new Error(`minerva failed: ${error}`);
  }

  return { incremented: true };
}
