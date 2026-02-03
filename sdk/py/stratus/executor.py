"""
Query executor

Handles prepared statement execution with both positional and named parameters.
"""

from typing import TypeVar, Generic, Dict, Any, List, Optional, Union
from dataclasses import dataclass

from .pool import StratusPool, PoolConnection
from .types import QueryResult, SingleResult, ExecuteOptions
from .params import (
    detect_param_style,
    parse_named_params,
    validate_param_count,
    prepare_params,
)


T = TypeVar("T")


@dataclass
class QueryInfo:
    """Query metadata"""

    sql: str
    param_count: int
    return_type: str
    param_names: Optional[List[str]] = None


def execute(pool: StratusPool) -> "StratusExecutor":
    """
    Create a query executor bound to a pool

    Args:
        pool: StratusPool instance

    Returns:
        StratusExecutor

    Example:
        ```python
        from stratus import create_pool, execute

        pool = create_pool("postgresql://...")
        exec = execute(pool)

        # Positional parameters (default - fastest)
        users = await exec.query(
            sql="SELECT * FROM users WHERE id = $1",
            params=[1],
            return_type="one",
        )

        # Named parameters (more readable)
        users = await exec.query(
            sql="SELECT * FROM users WHERE id = :id",
            params={"id": 1},
            return_type="one",
        )
        ```
    """
    return StratusExecutor(pool)


async def execute_query(
    pool: StratusPool,
    sql: str,
    params: Optional[Union[List[Any], Dict[str, Any]]] = None,
    *,
    return_type: str = "many",
    options: Optional[ExecuteOptions] = None,
) -> QueryResult:
    """
    Execute a single query (convenience function)

    Args:
        pool: Pool instance
        sql: SQL query
        params: Parameters (positional list or named dict)
        return_type: 'one', 'many', or 'exec'
        options: Execution options

    Returns:
        Query result
    """
    executor = StratusExecutor(pool)
    return await executor.query(
        sql=sql,
        params=params or [],
        return_type=return_type,
        options=options,
    )


async def execute_one(
    pool: StratusPool,
    sql: str,
    params: Optional[Union[List[Any], Dict[str, Any]]] = None,
    *,
    options: Optional[ExecuteOptions] = None,
) -> SingleResult:
    """
    Execute and return single row

    Args:
        pool: Pool instance
        sql: SQL query
        params: Parameters
        options: Execution options

    Returns:
        Single row result
    """
    executor = StratusExecutor(pool)
    return await executor.query_one(
        sql=sql,
        params=params or [],
        options=options,
    )


class StratusExecutor:
    """
    Stratus Query Executor

    Handles query execution with proper parameter handling.
    """

    def __init__(self, pool: StratusPool):
        self.pool = pool

    async def query(
        self,
        sql: str,
        params: Union[List[Any], Dict[str, Any]],
        *,
        return_type: str = "many",
        options: Optional[ExecuteOptions] = None,
    ) -> QueryResult:
        """
        Execute a query

        Args:
            sql: SQL query
            params: Parameters (positional list or named dict)
            return_type: 'one', 'many', or 'exec'
            options: Execution options

        Returns:
            Query result
        """
        param_style = detect_param_style(sql)
        param_array = prepare_params(sql, params)

        if param_style[0] != "unknown" and param_style[2] > 0:
            validate_param_count(sql, param_array, param_style[2])

        async with self.pool.connect() as conn:
            result = await conn.execute(sql, param_array)

            if return_type == "one" and result.row_count > 1:
                import warnings

                warnings.warn(
                    f"Query returned {result.row_count} rows but expected 1. "
                    f"Use return_type='many' for multiple rows."
                )

            return result

    async def query_one(
        self,
        sql: str,
        params: Union[List[Any], Dict[str, Any]],
        *,
        options: Optional[ExecuteOptions] = None,
    ) -> SingleResult:
        """
        Execute and return single row

        Args:
            sql: SQL query
            params: Parameters
            options: Execution options

        Returns:
            Single row result
        """
        result = await self.query(
            sql=sql,
            params=params,
            return_type="one",
            options=options,
        )

        return SingleResult(
            row=result.rows[0] if result.rows else None,
            found=result.row_count > 0,
        )

    async def exec(
        self,
        sql: str,
        params: Union[List[Any], Dict[str, Any]],
        *,
        options: Optional[ExecuteOptions] = None,
    ) -> QueryResult:
        """
        Execute without returning rows (INSERT, UPDATE, DELETE)

        Args:
            sql: SQL query
            params: Parameters
            options: Execution options

        Returns:
            Execution result
        """
        return await self.query(
            sql=sql,
            params=params,
            return_type="exec",
            options=options,
        )

    async def query_with_connection(
        self,
        sql: str,
        params: Union[List[Any], Dict[str, Any]],
        conn: PoolConnection,
        *,
        return_type: str = "many",
    ) -> QueryResult:
        """
        Execute with a dedicated connection (for transactions)

        Args:
            sql: SQL query
            params: Parameters
            conn: Pool connection
            return_type: 'one', 'many', or 'exec'

        Returns:
            Query result
        """
        param_style = detect_param_style(sql)
        param_array = prepare_params(sql, params)

        return await conn.execute(sql, param_array)
