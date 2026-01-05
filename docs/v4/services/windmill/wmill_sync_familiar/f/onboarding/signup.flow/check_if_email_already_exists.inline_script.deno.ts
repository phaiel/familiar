// Check if email already exists using Minerva CLI
// Replaces direct SQL: SELECT id FROM users WHERE email = $1

export async function main(email: string) {
  const cmd = new Deno.Command("minerva", {
    args: ["onboarding", "check-email", "--email", email],
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    throw new Error(`minerva failed: ${error}`);
  }

  const result = JSON.parse(new TextDecoder().decode(stdout));
  
  if (result.exists) {
    throw new Error(JSON.stringify({ 
      code: 'EMAIL_EXISTS', 
      message: 'An account with this email already exists' 
    }));
  }
  
  return { exists: false, email };
}
