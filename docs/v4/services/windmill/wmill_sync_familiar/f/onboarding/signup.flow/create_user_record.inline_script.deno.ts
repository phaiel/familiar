// Create user record using Minerva CLI
// Replaces direct SQL: INSERT INTO users (email, name, password_hash, ...)

export async function main(email: string, name: string, password_hash: string | null) {
  const args = [
    "onboarding", "create-user",
    "--email", email,
    "--name", name,
  ];
  
  if (password_hash) {
    args.push("--password-hash", password_hash);
  }

  const cmd = new Deno.Command("minerva", {
    args,
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await cmd.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    throw new Error(`minerva failed: ${error}`);
  }

  return JSON.parse(new TextDecoder().decode(stdout));
}
