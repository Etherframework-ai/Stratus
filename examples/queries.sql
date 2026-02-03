# name: GetUser :one id: number
SELECT id, name, email, created_at FROM users WHERE id = $1;

# name: ListUsers :many limit: number offset: number
SELECT id, name, email FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2;

# name: CreateUser :one name: string email: string
INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email;

# name: UpdateUser :one id: number name: string
UPDATE users SET name = $2 WHERE id = $1 RETURNING id, name, email;

# name: DeleteUser :one id: number
DELETE FROM users WHERE id = $1 RETURNING id;

# name: GetUserPosts :many user_id: number
SELECT id, title, content, created_at FROM posts WHERE user_id = $1 ORDER BY created_at DESC;
