export declare class AppError extends Error {
    code: string;
    status: number;
    constructor(code: string, message: string, status?: number);
}
export declare const ERRORS: {
    readonly VALIDATION_ERROR: (msg: string) => AppError;
    readonly NOT_FOUND: (msg: string) => AppError;
    readonly UNAUTHORIZED: (msg?: string) => AppError;
    readonly FORBIDDEN: (msg?: string) => AppError;
    readonly RATE_LIMITED: (msg?: string) => AppError;
    readonly INTERNAL_ERROR: (msg?: string) => AppError;
};
//# sourceMappingURL=errors.d.ts.map