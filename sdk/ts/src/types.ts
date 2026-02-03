/**
 * Configuration types for Stratus SDK
 */

/**
 * Stratus connection pool configuration
 */
export interface StratusConfig {
  /** Maximum number of clients in the pool */
  max: number;
  /** Idle timeout in milliseconds */
  idleTimeoutMillis: number;
  /** Connection timeout in milliseconds */
  connectionTimeoutMillis: number;
  /** PostgreSQL connection string */
  connectionString?: string;
}

/**
 * Query execution options
 */
export interface ExecuteOptions {
  /** Whether to use prepared statements (default: true) */
  prepared?: boolean;
  /** Query timeout in milliseconds (default: 30000) */
  timeout?: number;
}

/**
 * Transaction options
 */
export interface TransactionOptions {
  /** Isolation level */
  isolationLevel?: 'read uncommitted' | 'read committed' | 'repeatable read' | 'serializable';
  /** Read-only transaction */
  readOnly?: boolean;
  /** Deferrable constraint (PostgreSQL only) */
  deferrable?: boolean;
}

/**
 * Query result type
 */
export interface QueryResult<T = unknown> {
  /** Array of rows */
  rows: T[];
  /** Number of rows affected */
  rowCount: number;
  /** Command tag (e.g., "INSERT 0 1") */
  command: string;
  /** OIDs for inserted rows (if applicable) */
  oids?: number[];
}

/**
 * Single row result (for :one queries)
 */
export interface SingleResult<T = unknown> {
  /** The single row or null if not found */
  row: T | null;
  /** Whether a row was found */
  found: boolean;
}

/**
 * Parameter types supported by the SDK
 */
export type ParamValue = 
  | string 
  | number 
  | boolean 
  | null 
  | undefined 
  | Date 
  | Buffer 
  | Uint8Array
  | ParamValue[]
  | { [key: string]: ParamValue };

/**
 * Query metadata extracted from SQL
 */
export interface QueryMeta {
  /** SQL text with parameters replaced by placeholders */
  sql: string;
  /** Number of parameters expected */
  paramCount: number;
  /** Whether this is a RETURNING query */
  hasReturning: boolean;
  /** Expected return type: 'one', 'many', or 'exec' */
  returnType: 'one' | 'many' | 'exec';
}
