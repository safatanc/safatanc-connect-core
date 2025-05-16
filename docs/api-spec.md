# Safatanc Connect Core API Specification

This document outlines the API endpoints available in the Safatanc Connect Core application.

## Base URL

```
https://api.safatanc-connect.com/v1
```

## Authentication

The API uses JWT tokens for authentication. Most endpoints require a valid access token to be included in the Authorization header:

```
Authorization: Bearer <access_token>
```

## Response Format

All API responses follow a standard format:

```json
{
  "success": true|false,
  "message": "Optional message string",
  "data": { ... } // Optional data object
}
```

Success responses will have `success: true` and include data.
Error responses will have `success: false` and include an error message.

## Error Codes

| Status Code | Description                                  |
| ----------- | -------------------------------------------- |
| 400         | Bad Request - Invalid input parameters       |
| 401         | Unauthorized - Authentication required       |
| 403         | Forbidden - Insufficient permissions         |
| 404         | Not Found - Resource not found               |
| 409         | Conflict - Resource already exists           |
| 429         | Too Many Requests - Rate limit exceeded      |
| 500         | Internal Server Error - Something went wrong |

## Pagination

Many endpoints that return lists support pagination through query parameters:

- `page`: Page number (default: 1)
- `limit`: Items per page (default: 10)

Paginated responses include metadata:

```json
{
  "success": true,
  "data": {
    "data": [...],
    "total": 100,
    "page": 1,
    "limit": 10,
    "total_pages": 10
  }
}
```

## API Endpoints

### Authentication

#### Register a new user

```
POST /auth/register
```

**Request Body:**
```json
{
  "email": "user@example.com",
  "username": "username",
  "password": "StrongPassword123!",
  "full_name": "User Full Name",
  "avatar_url": "https://example.com/avatar.jpg"
}
```

**Response:** `201 Created`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "username",
    "full_name": "User Full Name",
    "avatar_url": "https://example.com/avatar.jpg",
    "global_role": "USER",
    "is_email_verified": false,
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Login with email/password

```
POST /auth/login
```

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "StrongPassword123!"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "username": "username",
      "full_name": "User Full Name",
      "avatar_url": "https://example.com/avatar.jpg",
      "global_role": "USER",
      "is_email_verified": true,
      "created_at": "2023-01-01T00:00:00Z"
    },
    "token": "jwt-token",
    "refresh_token": "refresh-token"
  }
}
```

#### Refresh access token

```
POST /auth/refresh
```

**Request Body:**
```json
{
  "refresh_token": "valid-refresh-token"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "token": "new-jwt-token"
  }
}
```

#### Logout

```
POST /auth/logout
```

**Authorization Required:** Yes

**Request Body:**
```json
{
  "refresh_token": "valid-refresh-token"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": "Logged out successfully"
}
```

#### Get current user

```
GET /auth/me
```

**Authorization Required:** Yes

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "username",
    "full_name": "User Full Name",
    "avatar_url": "https://example.com/avatar.jpg",
    "global_role": "USER",
    "is_email_verified": true,
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Verify Email

```
GET /auth/verify-email/:token
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "username",
    "full_name": "User Full Name",
    "avatar_url": "https://example.com/avatar.jpg",
    "global_role": "USER",
    "is_email_verified": true,
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Request Password Reset

```
POST /auth/request-password-reset
```

**Request Body:**
```json
{
  "email": "user@example.com"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": "Password reset link sent if the email exists in our system"
}
```

#### Reset Password

```
POST /auth/reset-password
```

**Request Body:**
```json
{
  "token": "password-reset-token",
  "new_password": "NewStrongPassword123!"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": "Password reset successfully"
}
```

#### OAuth Login

```
GET /auth/oauth/:provider
```

**Parameters:**
- `provider`: OAuth provider (e.g., "google", "github")

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "url": "https://oauth-provider.com/auth?client_id=xxx&redirect_uri=xxx"
  }
}
```

#### OAuth Callback

```
GET /auth/oauth/:provider/callback
```

**Query Parameters:**
- `code`: Authorization code from provider
- `state`: State parameter for security verification

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "username": "username",
      "full_name": "User Full Name",
      "avatar_url": "https://example.com/avatar.jpg",
      "global_role": "USER",
      "is_email_verified": true,
      "created_at": "2023-01-01T00:00:00Z"
    },
    "token": "jwt-token",
    "refresh_token": "refresh-token"
  }
}
```

### User Management

#### Get All Users (Admin only)

```
GET /users
```

**Authorization Required:** Yes (Admin role)

**Query Parameters:**
- `page`: Page number (default: 1)
- `limit`: Items per page (default: 10)

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "data": [
      {
        "id": "uuid",
        "email": "user@example.com",
        "username": "username",
        "full_name": "User Full Name",
        "avatar_url": "https://example.com/avatar.jpg",
        "global_role": "USER",
        "is_email_verified": true,
        "created_at": "2023-01-01T00:00:00Z"
      }
    ],
    "total": 100,
    "page": 1,
    "limit": 10,
    "total_pages": 10
  }
}
```

#### Create User (Admin only)

```
POST /users
```

**Authorization Required:** Yes (Admin role)

**Request Body:**
```json
{
  "email": "newuser@example.com",
  "username": "newusername",
  "password": "StrongPassword123!",
  "full_name": "New User",
  "avatar_url": "https://example.com/avatar.jpg"
}
```

**Response:** `201 Created`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "newuser@example.com",
    "username": "newusername",
    "full_name": "New User",
    "avatar_url": "https://example.com/avatar.jpg",
    "global_role": "USER",
    "is_email_verified": false,
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Get User by ID

```
GET /users/:id
```

**Authorization Required:** Yes (Admin role or own account)

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "username",
    "full_name": "User Full Name",
    "avatar_url": "https://example.com/avatar.jpg",
    "global_role": "USER",
    "is_email_verified": true,
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Update User

```
PUT /users/:id
```

**Authorization Required:** Yes (Admin role or own account)

**Request Body:**
```json
{
  "username": "updated_username",
  "full_name": "Updated Name",
  "avatar_url": "https://example.com/new-avatar.jpg",
  "is_active": true
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "updated_username",
    "full_name": "Updated Name",
    "avatar_url": "https://example.com/new-avatar.jpg",
    "global_role": "USER",
    "is_email_verified": true,
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Get Current User

```
GET /users/me
```

**Authorization Required:** Yes

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "username",
    "full_name": "User Full Name",
    "avatar_url": "https://example.com/avatar.jpg",
    "global_role": "USER",
    "is_email_verified": true,
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Update Current User

```
PUT /users/me
```

**Authorization Required:** Yes

**Request Body:**
```json
{
  "username": "updated_username",
  "full_name": "Updated Name",
  "avatar_url": "https://example.com/new-avatar.jpg"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "updated_username",
    "full_name": "Updated Name",
    "avatar_url": "https://example.com/new-avatar.jpg",
    "global_role": "USER",
    "is_email_verified": true,
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Update User Password

```
PUT /users/:id/password
```

**Authorization Required:** Yes (Admin role or own account)

**Request Body:**
```json
{
  "current_password": "CurrentPassword123!",
  "new_password": "NewStrongPassword123!"
}
```

**Note:** Admin users don't need to provide the current_password when changing other users' passwords.

**Response:** `200 OK`
```json
{
  "success": true,
  "data": "Password updated successfully"
}
```

#### Update Current User Password

```
PUT /users/me/password
```

**Authorization Required:** Yes

**Request Body:**
```json
{
  "current_password": "CurrentPassword123!",
  "new_password": "NewStrongPassword123!"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": "Password updated successfully"
}
```

#### Delete User (Admin only)

```
DELETE /users/:id
```

**Authorization Required:** Yes (Admin role)

**Response:** `204 No Content`

### Badges

#### Get All Badges

```
GET /badges
```

**Query Parameters:**
- `page`: Page number (default: 1)
- `limit`: Items per page (default: 10)

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "data": [
      {
        "id": "uuid",
        "name": "Badge Name",
        "description": "Badge description",
        "image_url": "https://example.com/badge.png",
        "created_at": "2023-01-01T00:00:00Z"
      }
    ],
    "total": 100,
    "page": 1,
    "limit": 10,
    "total_pages": 10
  }
}
```

#### Get Badge by ID

```
GET /badges/:id
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "Badge Name",
    "description": "Badge description",
    "image_url": "https://example.com/badge.png",
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Create Badge (Admin only)

```
POST /badges
```

**Authorization Required:** Yes (Admin role)

**Request Body:**
```json
{
  "name": "New Badge",
  "description": "New badge description",
  "image_url": "https://example.com/new-badge.png"
}
```

**Response:** `201 Created`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "New Badge",
    "description": "New badge description",
    "image_url": "https://example.com/new-badge.png",
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Update Badge (Admin only)

```
PUT /badges/:id
```

**Authorization Required:** Yes (Admin role)

**Request Body:**
```json
{
  "name": "Updated Badge Name",
  "description": "Updated description",
  "image_url": "https://example.com/updated-badge.png"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "Updated Badge Name",
    "description": "Updated description",
    "image_url": "https://example.com/updated-badge.png",
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

#### Delete Badge (Admin only)

```
DELETE /badges/:id
```

**Authorization Required:** Yes (Admin role)

**Response:** `204 No Content`

#### Award Badge to User (Admin only)

```
POST /badges/award
```

**Authorization Required:** Yes (Admin role)

**Request Body:**
```json
{
  "user_id": "user-uuid",
  "badge_id": "badge-uuid"
}
```

**Response:** `201 Created`
```json
{
  "success": true,
  "data": "Badge awarded successfully"
}
```

#### Remove Badge from User (Admin only)

```
DELETE /badges/users/:user_id/badges/:badge_id
```

**Authorization Required:** Yes (Admin role)

**Response:** `204 No Content`

#### Get Users with Badge

```
GET /badges/:id/users
```

**Authorization Required:** Yes

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "badge": {
      "id": "uuid",
      "name": "Badge Name",
      "description": "Badge description",
      "image_url": "https://example.com/badge.png",
      "created_at": "2023-01-01T00:00:00Z"
    },
    "users": [
      {
        "id": "uuid",
        "email": "user@example.com",
        "username": "username",
        "full_name": "User Full Name",
        "avatar_url": "https://example.com/avatar.jpg",
        "global_role": "USER",
        "is_email_verified": true,
        "created_at": "2023-01-01T00:00:00Z"
      }
    ]
  }
}
```

#### Get User's Badges

```
GET /badges/users/:user_id
```

**Authorization Required:** Yes

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "username": "username",
      "full_name": "User Full Name",
      "avatar_url": "https://example.com/avatar.jpg",
      "global_role": "USER",
      "is_email_verified": true,
      "created_at": "2023-01-01T00:00:00Z"
    },
    "badges": [
      {
        "id": "uuid",
        "name": "Badge Name",
        "description": "Badge description",
        "image_url": "https://example.com/badge.png",
        "created_at": "2023-01-01T00:00:00Z"
      }
    ]
  }
}
```

#### Check if User Has Badge

```
GET /badges/users/:user_id/badges/:badge_id/check
```

**Authorization Required:** Yes

**Response:** `200 OK`
```json
{
  "success": true,
  "data": true
}
``` 