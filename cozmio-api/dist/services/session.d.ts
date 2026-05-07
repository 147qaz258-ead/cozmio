export declare function sendOtp(email: string): Promise<string>;
export declare function verifyOtp(email: string, code: string): Promise<{
    user: {
        id: string;
        name: string | null;
        email: string;
        role: "user" | "admin";
        webAccess: boolean;
        betaAccess: boolean;
        desktopAccess: boolean;
        hardwareAccess: boolean;
        inviteCode: string | null;
        createdAt: Date;
        updatedAt: Date;
    };
    sessionToken: string;
    expiresAt: Date;
}>;
export declare function validateSession(token: string): Promise<{
    id: string;
    name: string | null;
    email: string;
    role: "user" | "admin";
    webAccess: boolean;
    betaAccess: boolean;
    desktopAccess: boolean;
    hardwareAccess: boolean;
    inviteCode: string | null;
    createdAt: Date;
    updatedAt: Date;
} | null>;
export declare function destroySession(token: string): Promise<void>;
//# sourceMappingURL=session.d.ts.map