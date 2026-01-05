export async function main(user_id: string, email: string, name: string, session_id: string, session_token: string, session_expires_at: string, needs_family: boolean, joined_family_id: string | null, joined_family_name: string | null) {
  return { user_id, email, name, session_id, session_token, session_expires_at, is_new_user: true, needs_family, joined_family_id, joined_family_name };
}