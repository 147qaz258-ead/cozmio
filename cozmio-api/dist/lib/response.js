export function ok(data) {
    return { ok: true, data };
}
export function error(code, message, status = 400) {
    return {
        status,
        body: { ok: false, error: { code, message } },
    };
}
export function paginated(data, page, pageSize, total) {
    return {
        ok: true,
        data,
        pagination: { page, pageSize, total },
    };
}
//# sourceMappingURL=response.js.map