# @stratusdb/wasm

High-performance TypeSQL parser compiled to WebAssembly using Rust.

## Installation

```bash
npm install @stratusdb/wasm
```

## Requirements

- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) must be installed

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

## Usage

### ES Modules (modern browsers, webpack 5, vite)

```javascript
import init, { parse_typesql, validate_typesql, extract_tables, extract_columns, get_version } from '@stratusdb/wasm';

async function main() {
  // Initialize WASM module
  await init();

  console.log('Version:', get_version());

  // Parse TypeSQL
  const result = parse_typesql(`
# name: GetUser :one id: number
SELECT * FROM users WHERE id = $1;
  `);

  if (result.ok) {
    const parsed = JSON.parse(result.val);
    console.log('Query name:', parsed.queries[0].name);
    console.log('SQL:', parsed.queries[0].sql);
  } else {
    console.error('Parse error:', result.val);
  }
}

main();
```

### CommonJS (Node.js with --experimental-modules)

```javascript
const wasm = require('@stratusdb/wasm/pkg/stratus');

async function main() {
  await wasm.default.init();

  const result = wasm.parse_typesql(`
# name: GetUser :one id: number
SELECT * FROM users WHERE id = $1;
  `);

  console.log(JSON.parse(result.val));
}

main();
```

## API

### parse_typesql(input: string): Result<string>

Parse TypeSQL content and return JSON string of parsed queries.

```typescript
interface ParsedQuery {
  name: string;
  return_type: 'one' | 'many' | 'exec';
  sql: string;
  params: {
    name: string;
    type: string;
    ordinal: number;
  }[];
}

interface ParsedQueryFile {
  queries: ParsedQuery[];
}
```

### validate_typesql(input: string): boolean

Validate TypeSQL syntax without full parsing.

### extract_tables(sql: string): Result<string>

Extract table names from SQL query.

### extract_columns(sql: string): Result<string>

Extract column names from SELECT query.

### get_version(): string

Get WASM module version.

## Building from Source

```bash
cd sdk/wasm
npm run build       # Build for web (browser)
npm run build:node  # Build for Node.js
```

## Performance

The WASM parser provides near-native Rust-level performance for parsing TypeSQL:

| Operation | Regex (JS) | WASM (Rust) | Speedup |
|-----------|-------------|--------------|---------|
| Parse 1000 queries | ~50ms | ~5ms | **10x** |
| Extract tables | ~5ms | ~0.5ms | **10x** |

## Browser Support

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## Node.js Support

Node.js 16+ with native WASM support (no transpiler needed).

## License

MIT
