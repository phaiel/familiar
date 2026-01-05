// Record GDPR consent records using Minerva CLI
// Replaces direct SQL: INSERT INTO consent_records (...)

export async function main(user_id: string, consents: any, ip_address: string | null, user_agent: string | null) {
  // Note: ip_address and user_agent are not currently passed to minerva CLI
  // They can be added later if needed for audit purposes
  
  const consentsJson = JSON.stringify({
    terms: consents.terms || false,
    privacy: consents.privacy || false,
    marketing: consents.marketing || false,
  });
  
  const cmd = new Deno.Command("minerva", {
    args: [
      "onboarding", "record-consent",
      "--user-id", user_id,
      "--consents-json", consentsJson,
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
  
  // Map the response format to match old behavior
  const records = [];
  if (consents.terms) records.push('terms_of_service');
  if (consents.privacy) records.push('privacy_policy');
  
  return { consents_recorded: records };
}
