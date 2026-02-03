"""
Transaction support
"""

from typing import TypeVar, Generic, Callable, Awaitable, Optional
from dataclasses import dataclass

from .pool import StratusPool, PoolConnection
from .executor import StratusExecutor
from .types import TransactionOptions, QueryResult


T = TypeVar("T")

TransactionScope = Callable[["StratusTransaction"], Awaitable[T]]


class StratusTransaction:
    """
    Transaction context

    Provides transaction-aware query execution.
    """

    def __init__(self, connection: PoolConnection):
        self._connection = connection
        self._executor = StratusExecutor(
            # Create a dummy pool - we'll use the connection directly
            __import__("stratus").pool.StratusPool("")
        )
        self._committed = False
        self._rolled_back = False

    async def execute(self, sql: str, params: Optional[list] = None) -> QueryResult:
        """Execute query within transaction"""
        return await self._connection.execute(sql, params)

    async def fetch(self, sql: str, params: Optional[list] = None) -> list:
        """Fetch rows within transaction"""
        return await self._connection.fetch(sql, params)

    async def fetchrow(self, sql: str, params: Optional[list] = None) -> dict:
        """Fetch single row within transaction"""
        return await self._connection.fetchrow(sql, params)

    async def commit(self) -> None:
        """Commit transaction"""
        if self._committed or self._rolled_back:
            raise RuntimeError("Transaction already finished")
        await self._connection.execute("COMMIT")
        self._committed = True

    async def rollback(self) -> None:
        """Rollback transaction"""
        if self._committed:
            return
        await self._connection.execute("ROLLBACK")
        self._rolled_back = True

    @property
    def committed(self) -> bool:
        return self._committed

    @property
    def rolled_back(self) -> bool:
        return self._rolled_back


async def with_transaction(
    pool: StratusPool,
    scope: TransactionScope[T],
    options: Optional[TransactionOptions] = None,
) -> T:
    """
    Run a function within a transaction

    Args:
        pool: Connection pool
        scope: Function to run in transaction
        options: Transaction options

    Returns:
        Result of scope function

    Raises:
        Exception: Re-raises any exception from scope after rollback

    Example:
        ```python
        from stratus import create_pool, with_transaction

        pool = create_pool("postgresql://...")

        result = await with_transaction(pool, async (trx) => {
            user = await trx.execute(
                "INSERT INTO users ... RETURNING *"
            )
            await trx.execute(
                "INSERT INTO orders ...",
                [user.rows[0]["id"]]
            )
            return user
        })
        ```
    """
    async with pool.connect() as connection:
        try:
            # Set isolation level if specified
            if options and options.isolation_level:
                await connection.execute(
                    f"SET TRANSACTION ISOLATION LEVEL {options.isolation_level.value}"
                )

            # Start transaction
            await connection.execute("BEGIN")

            # Create transaction context
            trx = StratusTransaction(connection)

            # Execute scope
            result = await scope(trx)

            # Commit if not already rolled back
            if not trx.rolled_back:
                await trx.commit()

            return result
        except Exception as error:
            # Rollback on any error
            try:
                await connection.execute("ROLLBACK")
            except Exception as rollback_error:
                # Log but don't throw - original error is more important
                print(f"Rollback failed: {rollback_error}")
            raise error


async def savepoint(
    trx: StratusTransaction,
    name: str,
) -> None:
    """Create a savepoint within current transaction"""
    await trx.execute(f"SAVEPOINT {name}")


async def rollback_to_savepoint(
    trx: StratusTransaction,
    name: str,
) -> None:
    """Rollback to a savepoint"""
    await trx.execute(f"ROLLBACK TO SAVEPOINT {name}")


async def release_savepoint(
    trx: StratusTransaction,
    name: str,
) -> None:
    """Release a savepoint"""
    await trx.execute(f"RELEASE SAVEPOINT {name}")
