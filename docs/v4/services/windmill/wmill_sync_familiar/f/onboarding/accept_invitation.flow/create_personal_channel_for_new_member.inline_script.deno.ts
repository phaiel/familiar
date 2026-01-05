// Create personal channel for new member using Minerva CLI
// Replaces direct SQL: INSERT INTO channels (...) with channel_type = 'personal'

export async function main(tenant_id: string, user_id: string, user_name: string) {
  const channelName = `${user_name}'s Channel`;
  
  const cmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "create-channel",
      "--tenant-id", tenant_id,
      "--channel-type", "personal",
      "--name", channelName,
      "--owner-id", user_id,
    ],
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
    channel_id: result.channel_id, 
    channel_name: result.name 
  };
}
