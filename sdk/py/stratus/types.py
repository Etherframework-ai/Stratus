"""
Type definitions for Stratus Python SDK
"""

from dataclasses import dataclass
from typing import Any, List, Optional, Dict, Union, Literal
from enum import Enum


@dataclass
class StratusConfig:
    """Stratus connection pool configuration"""

    max: int = 20
    idle_timeout_ms: int = 30000
    connection_timeout_ms: int = 5000
    connection_string: Optional[str] = None


@dataclass
class ExecuteOptions:
    """Query execution options"""

    prepared: bool = True
    timeout_ms: int = 30000


class IsolationLevel(Enum):
    """Transaction isolation levels"""

    READ_UNCOMMITTED = "READ UNCOMMITTED"
    READ_COMMITTED = "READ COMMITTED"
    REPEATABLE_READ = "REPEATABLE READ"
    SERIALIZABLE = "SERIALIZABLE"


@dataclass
class TransactionOptions:
    """Transaction options"""

    isolation_level: Optional[IsolationLevel] = None
    read_only: bool = False
    deferrable: bool = False


@dataclass
class QueryResult:
    """Query result container"""

    rows: List[Dict[str, Any]]
    row_count: int
    command: str


@dataclass
class SingleResult:
    """Single row result (for :one queries)"""

    row: Optional[Dict[str, Any]]
    found: bool


# Type aliases
ParamValue = Union[
    str,
    int,
    float,
    bool,
    None,
    List[Any],
    Dict[str, Any],
]
ParamsInput = Union[List[ParamValue], Dict[str, ParamValue]]
