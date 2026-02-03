/**
 * Stratus TypeScript SDK
 * 
 * Zero-ORM database SDK with connection pooling and type-safe query execution.
 * Supports both positional ($1, $2) and named (:id, :name) parameters.
 * 
 * @packageDocumentation
 */

// Re-exports for convenience
export { Pool, PoolClient, QueryResult } from 'pg';
export type { PoolConfig } from 'pg';

// Core exports
export { StratusPool } from './pool';
export { StratusExecutor, execute, executeOne } from './executor';
export { StratusTransaction, withTransaction } from './transaction';
export { parseNamedParams } from './params';

// Type exports
export type { StratusConfig, QueryResult as StratusQueryResult } from './types';

/**
 * Create a Stratus connection pool with default settings
 * 
 * @param connectionString - PostgreSQL connection string (e.g., postgresql://user:pass@host:5432/db)
 * @param options - Optional pool configuration
 * @returns StratusPool instance
 * 
 * @example
 * ```typescript
 * import { createPool } from '@stratusdb/sdk';
 * 
 * const pool = createPool(process.env.DATABASE_URL);
 * await pool.connect();
 * ```
 */
export function createPool(
  connectionString: string,
  options?: Partial<StratusConfig>
): StratusPool {
  return new StratusPool({
    connectionString,
    max: options?.max ?? 20,
    idleTimeoutMillis: options?.idleTimeoutMillis ?? 30000,
    connectionTimeoutMillis: options?.connectionTimeoutMillis ?? 5000,
  });
}

/**
 * Default pool configuration
 */
export const defaultConfig: StratusConfig = {
  max: 20,
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 5000,
};

/**
 * SDK version
 */
export const VERSION = '0.1.0';
