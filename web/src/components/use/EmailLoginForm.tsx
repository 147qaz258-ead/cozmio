"use client";

import { useState } from "react";
import { sendCode, verifyCode } from "@/lib/api";

interface Props {
  onSuccess: () => void;
}

export function EmailLoginForm({ onSuccess }: Props) {
  const [step, setStep] = useState<"email" | "code">("email");
  const [email, setEmail] = useState("");
  const [code, setCode] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleEmailSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await sendCode(email);
      setStep("code");
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleCodeSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await verifyCode(email, code);
      onSuccess();
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={step === "email" ? handleEmailSubmit : handleCodeSubmit}>
      {step === "email" ? (
        <div className="space-y-4">
          <input
            type="email"
            value={email}
            onChange={e => setEmail(e.target.value)}
            placeholder="输入邮箱地址"
            className="w-full px-4 py-3 border border-warm-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            required
          />
          <button
            type="submit"
            disabled={loading}
            className="w-full py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {loading ? "发送中..." : "继续"}
          </button>
        </div>
      ) : (
        <div className="space-y-4">
          <p className="text-sm text-warm-600">请查收邮件中的验证码</p>
          <input
            type="text"
            value={code}
            onChange={e => setCode(e.target.value)}
            placeholder="000000"
            className="w-full px-4 py-3 border border-warm-200 rounded-lg text-center text-2xl tracking-widest focus:outline-none focus:ring-2 focus:ring-primary-500"
            maxLength={6}
            required
          />
          <button
            type="submit"
            disabled={loading}
            className="w-full py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {loading ? "验证中..." : "确认"}
          </button>
          <button
            type="button"
            onClick={() => setStep("email")}
            className="w-full text-sm text-warm-500 hover:text-warm-700"
          >
            使用其他邮箱
          </button>
        </div>
      )}
      {error && <p className="mt-2 text-sm text-red-600">{error}</p>}
    </form>
  );
}