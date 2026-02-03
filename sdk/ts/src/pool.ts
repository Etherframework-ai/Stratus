/**
 * Connection pool management using pg
 */

import { Pool, PoolConfig, PoolClient } from 'pg';
import { StratusConfig } from './types';

/**
 * Stratus connection pool wrapper
 * 
 * Provides a higher-level interface over pg's Pool with:
 * - Automatic configuration
 * - Health checks
 * - Statistics tracking
 * 
 * @example
 * ```typescript
 * import { StratusPool } from '@stratusdb/sdk';
 * 
 * const pool = new StratusPool({
 *   connectionString: 'postgresql://user:pass@localhost:5432/db',
 *   max: 10,
 * });
 * 
 * const client = await pool.connect();
 * try {
 *   const result = await client.query('SELECT * FROM users');
 *   console.log(result.rows);
 * } finally {
 *   client.release();
 * }
 * ```
 */
export class StratusPool {
  private pool: Pool;
  private config: StratusConfig;
  
  /**
   * Create a new StratusPool
   * 
   * @param config - Pool configuration
   */
  constructor(config: Partial<StratusConfig> & { connectionString: string }) {
    this.config = {
      max: config.max ?? 20,
      idleTimeoutMillis: config.idleTimeoutMillis ?? 30000,
      connectionTimeoutMillis: config.connectionTimeoutMillis ?? 5000,
      connectionString: config.connectionString,
    };
    
    const pgConfig: PoolConfig = {
      connectionString: this.config.connectionString,
      max: this.config.max,
      idleTimeoutMillis: this.config.idleTimeoutMillis,
      connectionTimeoutMillis: this.config.connectionTimeoutMillis,
    };
    
    this.pool = new Pool(pgConfig);
    
    // Set up error handlers
    this.pool.on('error', (err) => {
      console.error('Unexpected error on idle client', err);
    });
  }
  
  /**
   * Get a client from the pool
   */
  async connect(): Promise<PoolClient> {
    return this.pool.connect();
  }
  
  /**
   * Execute a query using a client from the pool
   * 
   * @param sql - SQL query
   * @param params - Query parameters
   * @returns Query result
   */
  async query<T = unknown>(
    sql: string,
    params?: unknown[]
  ): Promise<{ rows: T[]; rowCount: number }> {
    const result = await this.pool.query(sql, params);
    return {
      rows: result.rows as T[],
      rowCount: result.rowCount ?? 0,
    };
  }
  
  /**
   * Execute a query with a dedicated client (for transactions)
   * 
   * @param sql - SQL query
   * @param params - Query parameters
   * @returns Query result
   */
  async queryWithClient<T = unknown>(
    sql: string,
    params?: unknown[]
  ): Promise<{ rows: T[]; rowCount: number }> {
    const client = await this.connect();
    try {
      const result = await client.query(sql, params);
      return {
        rows: result.rows as T[],
        rowCount: result.rowCount ?? 0,
      };
    } finally {
      client.release();
    }
  }
  
  /**
   * Get pool statistics
   */
  getStats(): {
    totalCount: number;
    idleCount: number;
    waitingCount: number;
  } {
    return {
      totalCount: this.pool.totalCount,
      idleCount: this.pool.idleCount,
      waitingCount: this.pool.waitingCount,
    };
  }
  
  /**
   * Check pool health
   */
  async healthCheck(): Promise<boolean> {
    try {
      await this.query('SELECT 1');
      return true;
    } catch {
      return false;
    }
  }
  
  /**
   * Close all connections in the pool
   */
  async end(): Promise<void> {
    await this.pool.end();
  }
  
  /**
   * Get the underlying pg Pool (for advanced use cases)
   */
  getUnderlyingPool(): Pool {
    return this.pool;
  }
}
