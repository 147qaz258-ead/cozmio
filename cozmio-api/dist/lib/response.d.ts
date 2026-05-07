export declare function ok<T>(data: T): {
    ok: boolean;
    data: T;
};
export declare function error(code: string, message: string, status?: number): {
    status: number;
    body: {
        ok: boolean;
        error: {
            code: string;
            message: string;
        };
    };
};
export declare function paginated<T>(data: T[], page: number, pageSize: number, total: number): {
    ok: boolean;
    data: T[];
    pagination: {
        page: number;
        pageSize: number;
        total: number;
    };
};
//# sourceMappingURL=response.d.ts.map