import { Resend } from "resend";

const resend = new Resend(process.env.RESEND_API_KEY);

export async function sendOtpEmail(email: string, code: string) {
  await resend.emails.send({
    from: "Cozmio <noreply@cozmio.net>",
    to: email,
    subject: "Your Cozmio verification code",
    html: `
      <div style="font-family: sans-serif; max-width: 480px; margin: 0 auto;">
        <h2 style="color: #151515;">Your verification code</h2>
        <p style="font-size: 24px; letter-spacing: 4px; font-weight: bold;">${code}</p>
        <p style="color: #625b54;">This code expires in 10 minutes. If you didn't request this, please ignore this email.</p>
      </div>
    `,
  });
}