// Create family chat channel using Minerva CLI
// Replaces direct SQL: INSERT INTO channels (...) with channel_type = 'family'

export async function main(tenant_id: string, family_name: string) {
  const channelName = `${family_name} Chat`;
  
  const cmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "create-channel",
      "--tenant-id", tenant_id,
      "--channel-type", "family",
      "--name", channelName,
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
    family_channel_id: result.channel_id, 
    family_channel_name: result.name 
  };
}
