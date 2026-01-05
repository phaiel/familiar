export async function main(input: any) {
  const errors: string[] = [];
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!emailRegex.test(input.email)) errors.push('Invalid email format');
  if (input.password && input.password.length < 8) errors.push('Password must be at least 8 characters');
  if (!input.consents?.terms) errors.push('Terms of service must be accepted');
  if (!input.consents?.privacy) errors.push('Privacy policy must be accepted');
  if (errors.length > 0) throw new Error(JSON.stringify({ code: 'VALIDATION_ERROR', errors }));
  return {
    valid: true,
    email: input.email.toLowerCase().trim(),
    name: input.name.trim(),
    has_password: !!input.password,
    has_invite_code: !!input.invite_code,
    request_id: input.request_id
  };
}