export interface AdminInfo {
    email: string;
}
export declare function createAdminSession(email: string): Promise<{
    token: string;
    expiresAt: Date;
}>;
export declare function validateAdminSession(token: string): Promise<AdminInfo | null>;
export declare function destroyAdminSession(token: string): Promise<void>;
export declare function verifyAdminPassword(email: string, password: string): Promise<boolean>;
//# sourceMappingURL=admin-session.d.ts.map