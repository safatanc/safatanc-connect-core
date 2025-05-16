# Safatanc Connect Core API Specification

This document outlines the API endpoints available in the Safatanc Connect Core application.

## Base URL

```
https://connect-core.safatanc.com
```

## Authentication

The API uses JWT tokens for authentication. Most endpoints require a valid access token to be included in the Authorization header:

```
Authorization: Bearer <access_token>
```

### Email Verification

Many protected endpoints require email verification. Users can login without verifying their email, but will only have access to the `/auth/resend-verification-email` endpoint until they verify their email address. After verification, they gain access to all protected endpoints.

## CORS Configuration

The API has CORS (Cross-Origin Resource Sharing) enabled, which can be configured via environment variables:

```
CORS_ALLOWED_ORIGINS=http://localhost:3000,https://connect.safatanc.com
```

- Use a comma-separated list of allowed origins
- Use `*` to allow all origins (default if not specified)

The API allows the following HTTP methods across all endpoints:
- GET
- POST
- PUT
- DELETE
- OPTIONS

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

## Performance Optimizations

### Asynchronous Processing

The API implements several performance optimizations to ensure fast response times:

1. **Asynchronous Email Sending**: All email operations (verification emails, password reset emails) are processed in the background after the API responds to the client.

2. **Asynchronous User Updates**: The following operations are performed asynchronously:
   - Login timestamp updates
   - Email verification status updates
   - Password reset operations
   - Token usage marking

These optimizations ensure that API responses are immediate, while database updates and email sending happen in the background.

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

**Note:** Upon successful registration, a verification email is automatically sent to the user's email address with instructions to verify their account. Email sending happens asynchronously and won't delay the API response.

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

**Note:** Login timestamp is updated asynchronously and won't delay the API response.

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

**Parameters:**
- `token`: Verification token received in the email

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

**Note:** Email verification status is updated asynchronously and won't delay the API response.

#### Resend Verification Email

```
POST /auth/resend-verification-email
```

**Authorization Required:** Yes

**Response:** `200 OK`
```json
{
  "success": true,
  "data": "Verification email sent"
}
```

**Note:** This endpoint is used by logged-in users who haven't verified their email yet. The verification email will be sent to the email address associated with the authenticated user. Email sending happens asynchronously and won't delay the API response.

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

**Note:** If the email exists in the system, a password reset email will be sent to the user's email address with a link to reset their password. Email sending happens asynchronously and won't delay the API response.

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

**Note:** Password updates and token invalidation happen asynchronously and won't delay the API response.

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

**Response:** `302 Found` (Redirect)

This endpoint redirects the user to the frontend application with the authentication tokens appended as query parameters:

```
{frontend_url}/auth/callback?token={jwt-token}&refresh_token={refresh-token}
```

**Note:** Instead of returning a JSON response, this endpoint performs a redirect to the frontend application, passing the authentication tokens as query parameters for the client application to process.

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

**Note:** Upon successful user creation, a verification email is automatically sent to the user's email address. Email sending happens asynchronously and won't delay the API response.

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

## Email Configuration

The application sends transactional emails for various events like user registration, email verification, and password reset. Emails are sent asynchronously to improve API response times - the API will respond immediately while email sending happens in the background.

To enable email functionality, the following environment variables need to be configured:

- `SMTP_HOST`: SMTP server hostname (default: smtp.gmail.com)
- `SMTP_PORT`: SMTP server port (default: 587)
- `SMTP_USERNAME`: Username for SMTP authentication
- `SMTP_PASSWORD`: Password for SMTP authentication
- `SENDER_EMAIL`: Email address used as sender (default: noreply@safatanc-connect.com)
- `SENDER_NAME`: Name displayed as sender (default: Safatanc Connect)
- `FRONTEND_URL`: Base URL of the frontend application for email links
