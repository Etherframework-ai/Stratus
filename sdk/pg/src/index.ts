/**
 * Stratus pg SDK - High-performance PostgreSQL TypeSQL Executor
 *
 * Architecture:
 * - Uses WASM parser (Rust) when available for maximum performance
 * - Falls back to optimized JavaScript regex parser
 * - Query caching for repeated executions
 *
 * @see {@link https://github.com/stratusdb/stratus/tree/main/sdk/wasm | @stratusdb/wasm}
 *
 * @example
 * ```typescript
 * import { Pool } from 'pg';
 * import { TypeSQLExecutor } from '@stratusdb/pg';
 *
 * // Load WASM parser for best performance
 * import('@stratusdb/wasm').then(wasm => {
 *   wasm.init();
 *   globalThis.stratus = { parseTypesql: wasm.parse_typesql };
 * });
 *
 * const pool = new Pool({ connectionString });
 * const executor = new TypeSQLExecutor();
 *
 * const user = await executor.query(pool)`
 *   # name: GetUser :one id: number
 *   SELECT * FROM users WHERE id = ${1}
 * `({ id: 1 });
 * ```
 */

import { Pool, PoolClient, QueryResultRow } from 'pg';

// Global type augmentation for WASM parser
declare global {
  namespace NodeJS {
    interface Global {
      stratus?: {
        parseTypesql?: (sql: string) => string;
      };
    }
  }
  interface Window {
    stratus?: {
      parseTypesql?: (sql: string) => string;
    };
  }
}

// ==================== 类型定义 ====================

/** 查询结果 */
export interface TypeSQLResult<T = unknown> {
  rows: T[];
  rowCount: number;
  command: string;
}

/** 单行结果 */
export interface SingleResult<T = unknown> {
  row: T | null;
  found: boolean;
}

/** 解析后的查询 */
export interface ParsedQuery {
  sql: string;
  name: string;
  returnType: 'one' | 'many' | 'exec';
  params: ParamDefinition[];
  /** 缓存最后使用时间 */
  lastUsed?: number;
}

export interface ParamDefinition {
  name: string;
  type: 'number' | 'string' | 'boolean';
  position: number;
}

// ==================== 轻量模式：用户传入 Client ====================

/**
 * TypeSQL 执行器（轻量模式）
 * 
 * 用户传入自己的 pg Client/Pool，SDK 只负责解析和执行 TypeSQL
 * 
 * @example
 * ```typescript
 * import { Pool } from 'pg';
 * import { TypeSQLExecutor } from '@stratusdb/pg';
 * 
 * const pool = new Pool({ connectionString });
 * const executor = new TypeSQLExecutor(pool);
 * 
 * // 模板字面量
 * const user = await executor.query(pool)`
 *   # name: GetUser :one id: number
 *   SELECT * FROM users WHERE id = ${1}
 * `({ id: 1 });
 * ```
 */
export class TypeSQLExecutor {
  private parser: TypeSQLParser;

  constructor() {
    this.parser = new TypeSQLParser();
  }

  /**
   * 执行查询（模板字面量）
   */
  async query<T extends QueryResultRow = QueryResultRow>(
    client: Pool | PoolClient,
    strings: TemplateStringsArray,
    ...values: unknown[]
  ): Promise<TypeSQLResult<T>> {
    const sql = this.interpolate(strings, values);
    return this.execute<T>(client, sql);
  }

  /**
   * 执行查询（命名参数）
   */
  async execute<T extends QueryResultRow = QueryResultRow>(
    client: Pool | PoolClient,
    sql: string,
    params?: Record<string, unknown>
  ): Promise<TypeSQLResult<T>> {
    const parsed = this.parser.parse(sql);
    const paramArray = this.extractParams(parsed, params);
    const result = await client.query<T>(parsed.sql, paramArray);
    return {
      rows: result.rows,
      rowCount: result.rowCount ?? 0,
      command: result.command,
    };
  }

  /**
   * 查询单行
   */
  async one<T extends QueryResultRow = QueryResultRow>(
    client: Pool | PoolClient,
    strings: TemplateStringsArray,
    ...values: unknown[]
  ): Promise<SingleResult<T>> {
    const result = await this.query<T>(client, strings, ...values);
    return {
      row: result.rows[0] ?? null,
      found: result.rowCount > 0,
    };
  }

  /**
   * 执行（无返回行）
   */
  async exec(
    client: Pool | PoolClient,
    strings: TemplateStringsArray,
    ...values: unknown[]
  ): Promise<TypeSQLResult<Record<string, unknown>>> {
    const sql = this.interpolate(strings, values);
    return this.execute(client, sql);
  }

  /**
   * 获取解析器（用于调试）
   */
  getParser(): TypeSQLParser {
    return this.parser;
  }

  private interpolate(
    strings: TemplateStringsArray,
    values: unknown[]
  ): string {
    let result = '';
    for (let i = 0; i < strings.length; i++) {
      result += strings[i];
      if (i < values.length) {
        const value = values[i];
        if (typeof value === 'number') {
          result += `$${value}`;
        } else if (typeof value === 'string') {
          result += `'${value.replace(/'/g, "''")}'`;
        } else if (value === null || value === undefined) {
          result += 'NULL';
        } else {
          result += String(value);
        }
      }
    }
    return result;
  }

  private extractParams(
    parsed: ParsedQuery,
    params?: Record<string, unknown>
  ): unknown[] {
    return parsed.params.map((p) => params?.[p.name] ?? null);
  }
}

// ==================== 完整模式：SDK 内置连接池 ====================

/**
 * SDK 内置连接池配置（代码方式）
 */
export interface StratusConfig {
  connectionString: string;
  max?: number;
  idleTimeoutMillis?: number;
  connectionTimeoutMillis?: number;
}

/**
 * 从 stratus.json 读取的配置
 */
export interface StratusJsonConfig {
  stratus: {
    version: number;
    datasources: {
      [name: string]: {
        url: string;
        schemas?: string[];
      };
    };
    schema?: {
      path?: string;
    };
    migrations?: {
      path?: string;
      auto_create?: boolean;
    };
  };
}

/**
 * Stratus SDK（完整模式）
 * 
 * SDK 内置 pg 连接池，开箱即用
 * 支持两种配置方式：
 * 1. 代码配置：直接传入连接字符串
 * 2. 配置文件：从 stratus.json 读取
 * 
 * @example
 * ```typescript
 * import { Stratus } from '@stratusdb/pg';
 * 
 * // 方式1：代码配置
 * const stratus1 = new Stratus({
 *   connectionString: process.env.DATABASE_URL,
 *   max: 20,
 * });
 * 
 * // 方式2：从 stratus.json 读取
 * const stratus2 = new Stratus({
 *   configPath: './stratus.json',
 *   datasource: 'primary',  // 使用哪个 datasource
 * });
 * 
 * // 执行查询
 * const user = await stratus.query`
 *   # name: GetUser :one id: number
 *   SELECT * FROM users WHERE id = ${1}
 * `({ id: 1 });
 * 
 * // 关闭连接池
 * await stratus.end();
 * ```
 */
export class Stratus {
  private pool: Pool;
  private executor: TypeSQLExecutor;
  private configPath?: string;
  private datasourceName?: string;

  constructor(config: StratusConfig | { configPath: string; datasource?: string }) {
    // 判断是代码配置还是文件配置
    if ('configPath' in config) {
      this.configPath = config.configPath;
      this.datasourceName = config.datasource || 'primary';
      
      // 从配置文件读取
      const fileConfig = this.loadConfig(config.configPath);
      const ds = this.getDatasource(fileConfig, this.datasourceName);
      
      this.pool = new Pool({
        connectionString: ds.url,
        max: 20,
        idleTimeoutMillis: 30000,
        connectionTimeoutMillis: 5000,
      });
    } else {
      // 代码配置
      this.pool = new Pool({
        connectionString: config.connectionString,
        max: config.max ?? 20,
        idleTimeoutMillis: config.idleTimeoutMillis ?? 30000,
        connectionTimeoutMillis: config.connectionTimeoutMillis ?? 5000,
      });
    }
    
    this.executor = new TypeSQLExecutor();
  }

  /**
   * 从文件加载配置
   */
  private loadConfig(configPath: string): StratusJsonConfig {
    try {
      const fs = require('fs');
      const content = fs.readFileSync(configPath, 'utf-8');
      return JSON.parse(content);
    } catch (e) {
      throw new Error(`Failed to load stratus.json from ${configPath}: ${e}`);
    }
  }

  /**
   * 获取 datasource 配置
   */
  private getDatasource(
    config: StratusJsonConfig, 
    name: string
  ): { url: string } {
    const ds = config.stratus?.datasources?.[name];
    if (!ds) {
      throw new Error(`Datasource '${name}' not found in stratus.json`);
    }
    return { url: ds.url };
  }

  /**
   * 执行查询（模板字面量）
   */
  async query<T extends QueryResultRow = QueryResultRow>(
    strings: TemplateStringsArray,
    ...values: unknown[]
  ): Promise<TypeSQLResult<T>> {
    return this.executor.query<T>(this.pool, strings, ...values);
  }

  /**
   * 执行查询（命名参数）
   */
  async execute<T extends QueryResultRow = QueryResultRow>(
    sql: string,
    params?: Record<string, unknown>
  ): Promise<TypeSQLResult<T>> {
    return this.executor.execute<T>(this.pool, sql, params);
  }

  /**
   * 查询单行
   */
  async one<T extends QueryResultRow = QueryResultRow>(
    strings: TemplateStringsArray,
    ...values: unknown[]
  ): Promise<SingleResult<T>> {
    return this.executor.one<T>(this.pool, strings, ...values);
  }

  /**
   * 执行（无返回行）
   */
  async exec(
    strings: TemplateStringsArray,
    ...values: unknown[]
  ): Promise<TypeSQLResult<Record<string, unknown>>> {
    return this.executor.exec(this.pool, strings, ...values);
  }

  /**
   * 获取原始 pg pool（高级用户）
   */
  getPool(): Pool {
    return this.pool;
  }

  /**
   * 获取执行器（高级用户）
   */
  getExecutor(): TypeSQLExecutor {
    return this.executor;
  }

  /**
   * 获取当前配置信息
   */
  getConfig(): { configPath?: string; datasource?: string } {
    return {
      configPath: this.configPath,
      datasource: this.datasourceName,
    };
  }

  /**
   * 关闭连接池
   */
  async end(): Promise<void> {
    await this.pool.end();
  }
}

// ==================== TypeSQL 解析器 ====================

/**
 * TypeSQL 解析器（带缓存）
 */
export class TypeSQLParser {
  private cache = new Map<string, ParsedQuery>();
  private cacheMaxAge = 5 * 60 * 1000; // 5 分钟
  private cacheMaxSize = 1000;

  parse(sql: string): ParsedQuery {
    const cached = this.cache.get(sql);
    if (cached && cached.lastUsed && Date.now() - cached.lastUsed < this.cacheMaxAge) {
      cached.lastUsed = Date.now();
      return cached;
    }

    const parsed = this.doParse(sql);
    this.cache.set(sql, { ...parsed, lastUsed: Date.now() });

    if (this.cache.size > this.cacheMaxSize) {
      this.cleanCache();
    }

    return parsed;
  }

  private doParse(sql: string): Omit<ParsedQuery, 'lastUsed'> {
    // Try WASM parser first if available
    const stratusModule = (globalThis as unknown as { stratus?: { parseTypesql?: (sql: string) => string } }).stratus;
    if (stratusModule?.parseTypesql) {
      try {
        const result = stratusModule.parseTypesql(sql);
        const parsed = JSON.parse(result);
        if (parsed.queries && parsed.queries.length > 0) {
          const q = parsed.queries[0];
          return {
            sql: q.sql,
            name: q.name,
            returnType: q.return_type,
            params: q.params.map((p: { name: string; type_: string; ordinal: number }) => ({
              name: p.name,
              type: p.type_ as 'number' | 'string' | 'boolean',
              position: p.ordinal,
            })),
          };
        }
      } catch {
        // Fall through to JS parser
      }
    }

    // Optimized JS regex parser (fallback)
    const nameMatch = sql.match(/#\s*name:\s*(\w+)/);
    const name = nameMatch?.[1] || 'anonymous';

    const returnMatch = sql.match(/:(one|many|exec)/);
    const returnType = (returnMatch?.[1] as 'one' | 'many' | 'exec') || 'many';

    const paramRegex = /(\w+)\s*:\s*(number|string|boolean)/g;
    const params: ParamDefinition[] = [];

    let match: RegExpExecArray | null = paramRegex.exec(sql);
    while (match !== null) {
      params.push({
        name: match[1],
        type: match[2] as 'number' | 'string' | 'boolean',
        position: params.length + 1,
      });
      match = paramRegex.exec(sql);
    }

    const cleanSql = sql
      .replace(/#.*$/gm, '')
      .replace(/\s+/g, ' ')
      .trim();

    return {
      sql: cleanSql,
      name,
      returnType,
      params,
    };
  }

  private cleanCache(): void {
    const now = Date.now();
    const entries = Array.from(this.cache.entries())
      .filter(([_, v]) => v.lastUsed && now - v.lastUsed < this.cacheMaxAge)
      .sort((a, b) => (b[1].lastUsed ?? 0) - (a[1].lastUsed ?? 0))
      .slice(0, this.cacheMaxSize);

    this.cache.clear();
    for (const [key, value] of entries) {
      this.cache.set(key, value);
    }
  }

  getStats(): { size: number } {
    return { size: this.cache.size };
  }

  clear(): void {
    this.cache.clear();
  }
}

// ==================== 便捷函数 ====================

/**
 * 创建轻量执行器
 */
export function createExecutor(): TypeSQLExecutor {
  return new TypeSQLExecutor();
}

/**
 * 创建完整 SDK 实例
 */
export function createStratus(config: StratusConfig): Stratus {
  return new Stratus(config);
}

export const VERSION = '0.1.0';
