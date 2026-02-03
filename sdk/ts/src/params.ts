/**
 * Parameter parsing utilities
 * 
 * Handles both positional ($1, $2, ...) and named (:id, :name) parameters.
 * Positional is the default for maximum performance.
 */

import { ParamValue } from './types';

/**
 * Detect parameter style in SQL
 */
export type ParamStyle = 'positional' | 'named' | 'unknown';

/**
 * Parse SQL to detect parameter style and extract info
 * 
 * @param sql - SQL query with parameters
 * @returns Object with param style and metadata
 */
export function detectParamStyle(sql: string): {
  style: ParamStyle;
  paramNames: string[];
  paramCount: number;
} {
  // Check for positional parameters ($1, $2, etc.)
  const positionalMatch = sql.match(/\$(\d+)/g);
  if (positionalMatch) {
    const maxParam = Math.max(...positionalMatch.map(m => parseInt(m.slice(1))));
    return {
      style: 'positional',
      paramNames: positionalMatch.map(m => `$${m.slice(1)}`),
      paramCount: maxParam,
    };
  }
  
  // Check for named parameters (:id, :name, etc.)
  const namedMatch = sql.match(/:([a-zA-Z_][a-zA-Z0-9_]*)/g);
  if (namedMatch) {
    const uniqueNames = [...new Set(namedMatch.map(m => m.slice(1)))];
    return {
      style: 'named',
      paramNames: uniqueNames,
      paramCount: uniqueNames.length,
    };
  }
  
  return {
    style: 'unknown',
    paramNames: [],
    paramCount: 0,
  };
}

/**
 * Parse named parameters from object
 * 
 * Converts { id: 1, name: 'test' } to array [1, 'test'] in correct order
 * 
 * @param sql - SQL with named parameters (:id, :name)
 * @param params - Object with parameter values
 * @returns Array of values in positional order for database
 * 
 * @example
 * ```typescript
 * const sql = 'SELECT * FROM users WHERE id = :id AND name = :name';
 * const result = parseNamedParams(sql, { name: 'John', id: 1 });
 * // result: [1, 'John']
 * ```
 */
export function parseNamedParams(
  sql: string,
  params: Record<string, ParamValue>
): ParamValue[] {
  const paramOrder: string[] = [];
  const regex = /:([a-zA-Z_][a-zA-Z0-9_]*)/g;
  
  for (let match = regex.exec(sql); match !== null; match = regex.exec(sql)) {
    const name = match[1];
    if (!paramOrder.includes(name)) {
      paramOrder.push(name);
    }
  }
  
  // Build array in the order parameters appear in SQL
  return paramOrder.map(name => {
    if (!(name in params)) {
      throw new Error(`Missing parameter :${name}`);
    }
    return params[name];
  });
}

/**
 * Validate parameter count matches expected
 * 
 * @param sql - SQL query
 * @param params - Parameter array
 * @param expectedCount - Expected number of parameters
 * @throws Error if count doesn't match
 */
export function validateParamCount(
  sql: string,
  params: ParamValue[],
  expectedCount: number
): void {
  const actualCount = params.filter(p => p !== undefined).length;
  
  if (actualCount !== expectedCount) {
    throw new Error(
      `Parameter count mismatch: expected ${expectedCount}, got ${actualCount}\n` +
      `SQL: ${sql}\n` +
      `Params: ${JSON.stringify(params)}`
    );
  }
}

/**
 * Convert JavaScript values to PostgreSQL-compatible format
 * 
 * @param value - Value to convert
 * @returns PostgreSQL-compatible value
 */
export function toPgValue(value: ParamValue): ParamValue {
  if (value === undefined) {
    return null;
  }
  
  if (value instanceof Date) {
    return value.toISOString();
  }
  
  if (Array.isArray(value)) {
    return value.map(toPgValue);
  }
  
  if (value instanceof Uint8Array) {
    return Buffer.from(value);
  }
  
  return value;
}

/**
 * Prepare parameters for query execution
 * 
 * @param sql - SQL query
 * @param params - Parameters (positional array or named object)
 * @returns Prepared parameters array
 */
export function prepareParams(
  sql: string,
  params: ParamValue[] | Record<string, ParamValue>
): ParamValue[] {
  const { style, paramCount } = detectParamStyle(sql);
  
  if (style === 'named') {
    // params should be an object
    if (!params || typeof params !== 'object' || Array.isArray(params)) {
      throw new Error(
        'Named parameters require an object, e.g., { id: 1, name: "test" }'
      );
    }
    return parseNamedParams(sql, params as Record<string, ParamValue>).map(toPgValue);
  }
  
  // Positional parameters
  if (Array.isArray(params)) {
    return params.map(toPgValue);
  }
  
  // Object passed for positional query - convert to array
  throw new Error(
    'Positional parameters require an array, e.g., [1, "test"]'
  );
}
