# name: GetUserWithOrders :many id: number
SELECT users.*, orders.* FROM users JOIN orders ON users.id = orders.user_id WHERE users.id = $1;

# name: GetUserInfo :one id: number
SELECT users.id, users.email, users.username FROM users WHERE users.id = $1;

# name: GetOrderDetails :many user_id: number
SELECT 
    orders.id,
    orders.order_number,
    orders.total_amount,
    orders.status,
    users.email,
    users.username
FROM orders
JOIN users ON orders.user_id = users.id
WHERE orders.user_id = $1;
