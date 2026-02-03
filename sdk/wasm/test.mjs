// Test file for WASM parser
import init, { parse_typesql, validate_typesql, extract_tables, extract_columns, get_version } from './pkg/stratus';

async function runTests() {
  console.log('Initializing WASM module...');
  await init();
  console.log(`WASM version: ${get_version()}`);

  // Test parse_typesql
  console.log('\n--- Testing parse_typesql ---');
  const sql = `
# name: GetUser :one id: number
SELECT * FROM users WHERE id = $1;

# name: ListUsers :many
SELECT * FROM users ORDER BY created_at DESC;
`;

  const result = parse_typesql(sql);
  if (result.ok) {
    console.log('Parsed successfully:');
    console.log(JSON.parse(result.val).queries.map(q => ({
      name: q.name,
      returnType: q.return_type,
      sql: q.sql.substring(0, 50) + '...'
    })));
  } else {
    console.error('Parse failed:', result.val);
  }

  // Test validate_typesql
  console.log('\n--- Testing validate_typesql ---');
  const validSql = '# name: GetUser :one id: number\nSELECT * FROM users WHERE id = $1;';
  const invalidSql = 'SELECT * FROM users WHERE id = $1;';

  console.log('Valid SQL:', validate_typesql(validSql));
  console.log('Invalid SQL (no header):', validate_typesql(invalidSql));

  // Test extract_tables
  console.log('\n--- Testing extract_tables ---');
  const selectSql = 'SELECT u.id, u.email, o.order_number FROM users u JOIN orders o ON u.id = o.user_id WHERE u.id = $1';
  const tablesResult = extract_tables(selectSql);
  if (tablesResult.ok) {
    console.log('Tables found:', JSON.parse(tablesResult.val));
  }

  // Test extract_columns
  console.log('\n--- Testing extract_columns ---');
  const columnsResult = extract_columns('SELECT u.id, u.email, o.order_number FROM users u');
  if (columnsResult.ok) {
    console.log('Columns found:', JSON.parse(columnsResult.val));
  }

  console.log('\n✓ All tests completed!');
}

runTests().catch(console.error);
