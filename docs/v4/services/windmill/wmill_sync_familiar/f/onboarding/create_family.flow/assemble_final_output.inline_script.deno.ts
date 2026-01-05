export async function main(tenant_id: string, tenant_name: string, personal_channel_id: string, personal_channel_name: string, family_channel_id: string, family_channel_name: string) {
  return { tenant_id, tenant_name, personal_channel_id, personal_channel_name, family_channel_id, family_channel_name };
}