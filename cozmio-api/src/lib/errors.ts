export class AppError extends Error {
  constructor(
    public code: string,
    message: string,
    public status = 400
  ) {
    super(message);
  }
}

export const ERRORS = {
  VALIDATION_ERROR: (msg: string) => new AppError("VALIDATION_ERROR", msg, 400),
  NOT_FOUND: (msg: string) => new AppError("NOT_FOUND", msg, 404),
  UNAUTHORIZED: (msg = "Unauthorized") => new AppError("UNAUTHORIZED", msg, 401),
  FORBIDDEN: (msg = "Forbidden") => new AppError("FORBIDDEN", msg, 403),
  RATE_LIMITED: (msg = "Too many requests") => new AppError("RATE_LIMITED", msg, 429),
  INTERNAL_ERROR: (msg = "Internal server error") => new AppError("INTERNAL_ERROR", msg, 500),
} as const;