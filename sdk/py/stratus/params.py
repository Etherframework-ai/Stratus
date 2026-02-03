"""
Parameter parsing utilities

Handles both positional ($1, $2, ...) and named (:id, :name) parameters.
Positional is the default for maximum performance.
"""

import re
from typing import Dict, List, Any, Tuple, Literal


ParamStyle = Literal["positional", "named", "unknown"]


def detect_param_style(sql: str) -> Tuple[ParamStyle, List[str], int]:
    """
    Detect parameter style in SQL

    Args:
        sql: SQL query with parameters

    Returns:
        Tuple of (style, param_names, param_count)
    """
    # Check for positional parameters ($1, $2, etc.)
    positional_match = re.findall(r"\$(\d+)", sql)
    if positional_match:
        max_param = max(int(p) for p in positional_match)
        return ("positional", [f"${p}" for p in positional_match], max_param)

    # Check for named parameters (:id, :name, etc.)
    named_match = re.findall(r":([a-zA-Z_][a-zA-Z0-9_]*)", sql)
    if named_match:
        unique_names = list(
            dict.fromkeys(named_match)
        )  # Preserve order, remove duplicates
        return ("named", unique_names, len(unique_names))

    return ("unknown", [], 0)


def parse_named_params(sql: str, params: Dict[str, Any]) -> List[Any]:
    """
    Parse named parameters from object

    Converts {'id': 1, 'name': 'test'} to [1, 'test'] in SQL order

    Args:
        sql: SQL with named parameters (:id, :name)
        params: Dict with parameter values

    Returns:
        List of values in positional order for database

    Raises:
        ValueError: If a required parameter is missing
    """
    param_order: List[str] = []

    # Find all named parameters in order of appearance
    for match in re.finditer(r":([a-zA-Z_][a-zA-Z0-9_]*)", sql):
        name = match.group(1)
        if name not in param_order:
            param_order.append(name)

    # Build array in the order parameters appear in SQL
    result = []
    for name in param_order:
        if name not in params:
            raise ValueError(f"Missing parameter :{name}")
        result.append(_to_pg_value(params[name]))

    return result


def validate_param_count(sql: str, params: List[Any], expected_count: int) -> None:
    """
    Validate parameter count matches expected

    Args:
        sql: SQL query
        params: Parameter array
        expected_count: Expected number of parameters

    Raises:
        ValueError: If count doesn't match
    """
    actual_count = len([p for p in params if p is not None])

    if actual_count != expected_count:
        raise ValueError(
            f"Parameter count mismatch: expected {expected_count}, got {actual_count}\n"
            f"SQL: {sql}\n"
            f"Params: {params}"
        )


def _to_pg_value(value: Any) -> Any:
    """Convert Python value to PostgreSQL-compatible format"""
    if value is None:
        return None

    if isinstance(value, list):
        return [_to_pg_value(v) for v in value]

    if isinstance(value, dict):
        return {k: _to_pg_value(v) for k, v in value.items()}

    # Dates and other types can be passed directly to asyncpg
    return value


def prepare_params(sql: str, params: Any) -> List[Any]:
    """
    Prepare parameters for query execution

    Args:
        sql: SQL query
        params: Parameters (positional list or named dict)

    Returns:
        Prepared parameters array

    Raises:
        ValueError: If parameter style doesn't match
    """
    style, _, _ = detect_param_style(sql)

    if style == "named":
        if not isinstance(params, dict):
            raise ValueError(
                "Named parameters require a dict, e.g., {'id': 1, 'name': 'test'}"
            )
        return parse_named_params(sql, params)

    if style == "positional":
        if not isinstance(params, list):
            raise ValueError("Positional parameters require a list, e.g., [1, 'test']")
        return [_to_pg_value(p) for p in params]

    # No parameters
    return []
