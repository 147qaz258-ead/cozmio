export function ok<T>(data: T) {
  return { ok: true, data };
}

export function error(code: string, message: string, status = 400) {
  return {
    status,
    body: { ok: false, error: { code, message } },
  };
}

export function paginated<T>(data: T[], page: number, pageSize: number, total: number) {
  return {
    ok: true,
    data,
    pagination: { page, pageSize, total },
  };
}