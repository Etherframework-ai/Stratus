/**
 * Transaction support
 */

import { PoolClient } from 'pg';
import { StratusPool } from './pool';
import { StratusExecutor } from './ executor';
import { TransactionOptions } from './types';

/**
 * Transaction scope function
 */
export type TransactionScope<T> = (trx: StratusTransaction) => Promise<T>;

/**
 * Transaction context
 */
export class StratusTransaction {
  private client: PoolClient;
  private executor: StratusExecutor;
  private committed = false;
  private rolledBack = false;
  
  constructor(client: PoolClient) {
    this.client = client;
    this.executor = new StratusExecutor(new Proxy({} as StratusPool, {
      get: () => ({
        query: (sql: string, params?: unknown[]) => 
          this.client.query(sql, params) as Promise<{ rows: unknown[]; rowCount: number; command: string }>,
      }),
    }));
  }
  
  /**
   * Execute query within transaction
   */
  async query<T = unknown>(
    sql: string,
    params?: unknown[]
  ): Promise<{ rows: T[]; rowCount: number }> {
    const result = await this.client.query<T>(sql, params);
    return {
      rows: result.rows,
      rowCount: result.rowCount ?? 0,
    };
  }
  
  /**
   * Get the underlying client
   */
  getClient(): PoolClient {
    return this.client;
  }
  
  /**
   * Commit transaction
   */
  async commit(): Promise<void> {
    if (this.committed || this.rolledBack) {
      throw new Error('Transaction already finished');
    }
    await this.client.query('COMMIT');
    this.committed = true;
  }
  
  /**
   * Rollback transaction
   */
  async rollback(): Promise<void> {
    if (this.committed || this.rolledBack) {
      return;
    }
    await this.client.query('ROLLBACK');
    this.rolledBack = true;
  }
}

/**
 * Run a function within a transaction
 * 
 * @param pool - Connection pool
 * @param scope - Function to run in transaction
 * @param options - Transaction options
 * @returns Result of scope function
 * 
 * @example
 * ```typescript
 * import { createPool, withTransaction } from '@stratusdb/sdk';
 * 
 * const pool = createPool(process.env.DATABASE_URL!);
 * 
 * const result = await withTransaction(pool, async (trx) => {
 *   const user = await trx.query<User>('INSERT INTO users ... RETURNING *');
 *   await trx.query('INSERT INTO orders ...', [user.rows[0].id]);
 *   return user;
 * });
 * ```
 */
export async function withTransaction<T>(
  pool: StratusPool,
  scope: TransactionScope<T>,
  options?: TransactionOptions
): Promise<T> {
  const client = await pool.connect();
  
  try {
    // Set isolation level if specified
    if (options?.isolationLevel) {
      await client.query(`SET TRANSACTION ISOLATION LEVEL ${options.isolationLevel}`);
    }
    
    // Start transaction
    await client.query('BEGIN');
    
    // Create transaction context
    const trx = new StratusTransaction(client);
    
    // Execute scope
    const result = await scope(trx);
    
    // Commit if not already rolled back
    if (!trx.rolledBack) {
      await trx.commit();
    }
    
    return result;
  } catch (error) {
    // Rollback on any error
    try {
      await client.query('ROLLBACK');
    } catch (rollbackError) {
      // Log but don't throw - original error is more important
      console.error('Rollback failed:', rollbackError);
    }
    throw error;
  } finally {
    client.release();
  }
}

/**
 * Create a savepoint within current transaction
 * 
 * @param trx - Transaction instance
 * @param name - Savepoint name
 */
export async function savepoint(
  trx: StratusTransaction,
  name: string
): Promise<void> {
  await trx.query(`SAVEPOINT ${name}`);
}

/**
 * Rollback to a savepoint
 * 
 * @param trx - Transaction instance
 * @param name - Savepoint name
 */
export async function rollbackToSavepoint(
  trx: StratusTransaction,
  name: string
): Promise<void> {
  await trx.query(`ROLLBACK TO SAVEPOINT ${name}`);
}

/**
 * Release a savepoint
 * 
 * @param trx - Transaction instance
 * @param name - Savepoint name
 */
export async function releaseSavepoint(
  trx: StratusTransaction,
  name: string
): Promise<void> {
  await trx.query(`RELEASE SAVEPOINT ${name}`);
}
