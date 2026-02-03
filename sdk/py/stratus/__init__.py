"""
Stratus Python SDK

Zero-ORM database SDK with connection pooling and type-safe query execution.
Supports both positional ($1, $2) and named (:id, :name) parameters.
"""

from .pool import StratusPool, create_pool
from .executor import StratusExecutor, execute, execute_one
from .transaction import StratusTransaction, with_transaction
from .params import parse_named_params, detect_param_style
from .types import (
    StratusConfig,
    QueryResult,
    SingleResult,
    ExecuteOptions,
    TransactionOptions,
)

__version__ = "0.1.0"

__all__ = [
    # Pool
    "StratusPool",
    "create_pool",
    # Executor
    "StratusExecutor",
    "execute",
    "execute_one",
    # Transaction
    "StratusTransaction",
    "with_transaction",
    # Utils
    "parse_named_params",
    "detect_param_style",
    # Types
    "StratusConfig",
    "QueryResult",
    "SingleResult",
    "ExecuteOptions",
    "TransactionOptions",
]
