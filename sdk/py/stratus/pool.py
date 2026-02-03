"""
Connection pool management using asyncpg

Supports both asyncpg (preferred) and psycopg2.
"""

import asyncio
from typing import Optional, Dict, Any, AnyStr
from dataclasses import dataclass
from contextlib import asynccontextmanager

try:
    import asyncpg

    ASYNCPG_AVAILABLE = True
except ImportError:
    ASYNCPG_AVAILABLE = False

try:
    import psycopg
    import psycopg.pool

    PSYCOPG_AVAILABLE = True
except ImportError:
    PSYCOPG_AVAILABLE = False

from .types import StratusConfig, QueryResult


@dataclass
class PoolStats:
    """Pool statistics"""

    total_count: int
    idle_count: int
    waiting_count: int


class StratusPool:
    """
    Stratus connection pool wrapper

    Provides a higher-level interface over asyncpg/psycopg2 pool with:
    - Automatic configuration
    - Health checks
    - Statistics tracking

    Args:
        config: Pool configuration

    Example:
        ```python
        import asyncio
        from stratus import create_pool

        async def main():
            pool = create_pool("postgresql://user:pass@localhost:5432/db")

            async with pool.connect() as conn:
                result = await conn.fetch("SELECT * FROM users")
                print(result)

            await pool.close()

        asyncio.run(main())
        ```
    """

    def __init__(
        self,
        connection_string: str,
        *,
        max: int = 20,
        idle_timeout_ms: int = 30000,
        connection_timeout_ms: int = 5000,
    ):
        self.config = StratusConfig(
            max=max,
            idle_timeout_ms=idle_timeout_ms,
            connection_timeout_ms=connection_timeout_ms,
            connection_string=connection_string,
        )

        if ASYNCPG_AVAILABLE:
            self._pool: Optional[asyncpg.Pool] = None
            self._driver = "asyncpg"
        elif PSYCOPG_AVAILABLE:
            self._pool = psycopg.pool.ThreadedConnectionPool(
                "user",
                max,
                connection_string,
                connect_timeout=connection_timeout_ms / 1000,
            )
            self._driver = "psycopg2"
        else:
            raise RuntimeError(
                "No PostgreSQL driver available. "
                "Install asyncpg: pip install asyncpg "
                "or psycopg2: pip install psycopg2-binary"
            )

    async def _init_async_pool(self) -> None:
        """Initialize asyncpg pool"""
        if self._pool is None and ASYNCPG_AVAILABLE:
            self._pool = await asyncpg.create_pool(
                self.config.connection_string or "",
                min_size=1,
                max_size=self.config.max,
                idle_timeout=self.config.idle_timeout_ms / 1000,
                command_timeout=self.config.connection_timeout_ms / 1000,
            )

    @property
    def driver(self) -> str:
        """Get the driver being used"""
        return self._driver if hasattr(self, "_driver") else "unknown"

    async def connect(self) -> "PoolConnection":
        """Get a connection from the pool"""
        await self._init_async_pool()

        if self._driver == "asyncpg" and ASYNCPG_AVAILABLE:
            conn = await self._pool.acquire()
            return AsyncPGConnection(conn)
        else:
            conn = self._pool.getconn()
            return PsycopgConnection(conn)

    async def execute(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> QueryResult:
        """Execute a query and return results"""
        async with self.connect() as conn:
            return await conn.execute(sql, params)

    async def execute_one(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> Dict[str, Any]:
        """Execute a query and return single row"""
        async with self.connect() as conn:
            row = await conn.fetchrow(sql, params)
            return dict(row) if row else {}

    async def health_check(self) -> bool:
        """Check pool health"""
        try:
            await self.execute("SELECT 1")
            return True
        except Exception:
            return False

    def get_stats(self) -> PoolStats:
        """Get pool statistics"""
        if self._driver == "asyncpg" and self._pool:
            return PoolStats(
                total_count=self._pool.get_size(),
                idle_count=self._pool.get_idle_size(),
                waiting_count=self._pool.get_wait_size(),
            )
        elif self._driver == "psycopg2":
            return PoolStats(
                total_count=self._pool.maxsize,
                idle_count=len(self._pool.idle),
                waiting_count=0,
            )
        return PoolStats(total_count=0, idle_count=0, waiting_count=0)

    async def close(self) -> None:
        """Close all connections in the pool"""
        if self._driver == "asyncpg" and self._pool:
            await self._pool.close()
        elif self._driver == "psycopg2":
            self._pool.closeall()


class PoolConnection:
    """Abstract connection interface"""

    async def __aenter__(self) -> "PoolConnection":
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        await self.close()

    async def execute(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> QueryResult:
        raise NotImplementedError

    async def fetch(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> List[Dict[str, Any]]:
        raise NotImplementedError

    async def fetchrow(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> Dict[str, Any]:
        raise NotImplementedError

    async def close(self) -> None:
        raise NotImplementedError

    async def fetch(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> List[Dict[str, Any]]:
        raise NotImplementedError

    async def fetchrow(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> Dict[str, Any]:
        raise NotImplementedError

    async def close(self) -> None:
        raise NotImplementedError


class AsyncPGConnection(PoolConnection):
    """asyncpg connection wrapper"""

    def __init__(self, conn: asyncpg.Connection):
        self._conn = conn

    async def execute(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> QueryResult:
        result = await self._conn.fetch(sql, params)
        return QueryResult(
            rows=[dict(r) for r in result],
            row_count=len(result),
            command="SELECT",
        )

    async def fetch(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> List[Dict[str, Any]]:
        result = await self._conn.fetch(sql, params)
        return [dict(r) for r in result]

    async def fetchrow(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> Dict[str, Any]:
        row = await self._conn.fetchrow(sql, params)
        return dict(row) if row else {}

    async def close(self) -> None:
        await self._conn.close()


class PsycopgConnection(PoolConnection):
    """psycopg2 connection wrapper"""

    def __init__(self, conn):
        self._conn = conn

    def execute(self, sql: str, params: Optional[List[Any]] = None) -> QueryResult:
        with self._conn.cursor() as cur:
            cur.execute(sql, params)
            result = cur.fetchall()
            columns = [desc[0] for desc in cur.description] if cur.description else []
            rows = [dict(zip(columns, r)) for r in result]
            return QueryResult(
                rows=rows,
                row_count=cur.rowcount,
                command=cur.statusmessage or "",
            )

    def fetch(
        self, sql: str, params: Optional[List[Any]] = None
    ) -> List[Dict[str, Any]]:
        with self._conn.cursor() as cur:
            cur.execute(sql, params)
            result = cur.fetchall()
            columns = [desc[0] for desc in cur.description] if cur.description else []
            return [dict(zip(columns, r)) for r in result]

    def fetchrow(self, sql: str, params: Optional[List[Any]] = None) -> Dict[str, Any]:
        with self._conn.cursor() as cur:
            cur.execute(sql, params)
            row = cur.fetchone()
            if row:
                columns = [desc[0] for desc in cur.description]
                return dict(zip(columns, row))
            return {}

    def close(self) -> None:
        self._conn.close()


def create_pool(
    connection_string: str,
    *,
    max: int = 20,
    idle_timeout_ms: int = 30000,
    connection_timeout_ms: int = 5000,
) -> StratusPool:
    """
    Create a Stratus connection pool

    Args:
        connection_string: PostgreSQL connection string
        max: Maximum pool size
        idle_timeout_ms: Idle timeout in milliseconds
        connection_timeout_ms: Connection timeout in milliseconds

    Returns:
        StratusPool instance
    """
    return StratusPool(
        connection_string,
        max=max,
        idle_timeout_ms=idle_timeout_ms,
        connection_timeout_ms=connection_timeout_ms,
    )
