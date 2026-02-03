/**
 * Query executor
 * 
 * Handles prepared statement execution with both positional and named parameters.
 */

import { PoolClient } from 'pg';
import { StratusPool } from './pool';
import { ParamValue, QueryResult, SingleResult, ExecuteOptions } from './types';
import { detectParamStyle, parseNamedParams, validateParamCount, toPgValue } from './params';

/**
 * Query metadata for prepared statements
 */
export interface QueryInfo {
  /** Original SQL */
  sql: string;
  /** Parameter count */
  paramCount: number;
  /** Return type */
  returnType: 'one' | 'many' | 'exec';
  /** Parameter names (for debugging) */
  paramNames?: string[];
}

/**
 * Create a query executor bound to a pool
 * 
 * @param pool - StratusPool instance
 * @returns StratusExecutor
 * 
 * @example
 * ```typescript
 * import { createPool, execute } from '@stratusdb/sdk';
 * 
 * const pool = createPool(process.env.DATABASE_URL!);
 * const exec = execute(pool);
 * 
 * // Positional parameters (default - fastest)
 * const users = await exec.query({
 *   sql: 'SELECT * FROM users WHERE id = $1',
 *   params: [1],
 *   returnType: 'one',
 * });
 * 
 * // Named parameters (more readable)
 * const users = await exec.query({
 *   sql: 'SELECT * FROM users WHERE id = :id',
 *   params: { id: 1 },
 *   returnType: 'one',
 * });
 * ```
 */
export function execute(pool: StratusPool): StratusExecutor {
  return new StratusExecutor(pool);
}

/**
 * Execute a single query (convenience function)
 * 
 * @param pool - Pool instance
 * @param sql - SQL query
 * @param params - Parameters (positional array or named object)
 * @returns Query result
 */
export async function executeQuery<T = unknown>(
  pool: StratusPool,
  sql: string,
  params?: ParamValue[] | Record<string, ParamValue>,
  options?: ExecuteOptions
): Promise<QueryResult<T>> {
  const executor = new StratusExecutor(pool);
  return executor.query<T>({
    sql,
    params: params ?? [],
    returnType: 'many',
    options,
  });
}

/**
 * Execute and return single row
 * 
 * @param pool - Pool instance
 * @param sql - SQL query
 * @param params - Parameters
 * @returns Single row result
 */
export async function executeOne<T = unknown>(
  pool: StratusPool,
  sql: string,
  params?: ParamValue[] | Record<string, ParamValue>,
  options?: ExecuteOptions
): Promise<SingleResult<T>> {
  const executor = new StratusExecutor(pool);
  return executor.queryOne<T>({
    sql,
    params: params ?? [],
    returnType: 'one',
    options,
  });
}

/**
 * Stratus Query Executor
 */
export class StratusExecutor {
  private pool: StratusPool;
  
  constructor(pool: StratusPool) {
    this.pool = pool;
  }
  
  /**
   * Execute a query
   * 
   * @param query - Query info object
   * @returns Query result
   */
  async query<T = unknown>(query: {
    sql: string;
    params: ParamValue[] | Record<string, ParamValue>;
    returnType: 'one' | 'many' | 'exec';
    options?: ExecuteOptions;
  }): Promise<QueryResult<T>> {
    const { sql, params, returnType, options } = query;
    const paramStyle = detectParamStyle(sql);
    
    // Prepare parameters based on style
    let paramArray: ParamValue[];
    if (paramStyle.style === 'named') {
      paramArray = parseNamedParams(sql, params as Record<string, ParamValue>);
    } else {
      paramArray = params as ParamValue[];
    }
    
    // Validate parameter count
    if (paramStyle.paramCount > 0) {
      validateParamCount(sql, paramArray, paramStyle.paramCount);
    }
    
    // Execute query
    const result = await this.pool.query<T>(
      sql,
      paramArray.map(toPgValue)
    );
    
    // Validate result for :one queries
    if (returnType === 'one' && result.rowCount > 1) {
      console.warn(
        `Query returned ${result.rowCount} rows but expected 1. ` +
        `Use :many for multiple rows.`
      );
    }
    
    return result;
  }
  
  /**
   * Execute and return single row
   * 
   * @param query - Query info object
   * @returns Single row result
   */
  async queryOne<T = unknown>(query: {
    sql: string;
    params: ParamValue[] | Record<string, ParamValue>;
    options?: ExecuteOptions;
  }): Promise<SingleResult<T>> {
    const result = await this.query<T>({
      ...query,
      returnType: 'one',
    });
    
    return {
      row: result.rows[0] ?? null,
      found: result.rowCount > 0,
    };
  }
  
  /**
   * Execute without returning rows (INSERT, UPDATE, DELETE)
   * 
   * @param query - Query info object
   * @returns Execution result
   */
  async exec(query: {
    sql: string;
    params: ParamValue[] | Record<string, ParamValue>;
    options?: ExecuteOptions;
  }): Promise<QueryResult<null>> {
    return this.query<null>({
      ...query,
      returnType: 'exec',
    });
  }
  
  /**
   * Execute with a dedicated client (for transactions)
   * 
   * @param query - Query info object
   * @param client - Pool client
   * @returns Query result
   */
  async queryWithClient<T = unknown>(
    query: {
      sql: string;
      params: ParamValue[] | Record<string, ParamValue>;
      returnType: 'one' | 'many' | 'exec';
    },
    client: PoolClient
  ): Promise<QueryResult<T>> {
    const { sql, params, returnType } = query;
    const paramStyle = detectParamStyle(sql);
    
    let paramArray: ParamValue[];
    if (paramStyle.style === 'named') {
      paramArray = parseNamedParams(sql, params as Record<string, ParamValue>);
    } else {
      paramArray = params as ParamValue[];
    }
    
    const result = await client.query<T>(sql, paramArray.map(toPgValue));
    
    return {
      rows: result.rows,
      rowCount: result.rowCount ?? 0,
      command: result.command,
    };
  }
}
