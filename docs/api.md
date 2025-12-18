# API Reference

## Authentication

All API endpoints require authentication via the `tbl_token` cookie. Optional HTTP Basic auth can be enabled for additional security.

### Obtaining a Token

1. Start tbl — browser opens automatically
2. URL contains `?token=...`
3. JavaScript at `/bootstrap` sets cookie
4. Subsequent requests include cookie automatically

## HTTP Endpoints

### `GET /`

Root handler. Redirects to `/web/` if content exists, otherwise shows setup form.

**Response:**

- `302 Redirect` to `/web/` (if cloned)
- `200 OK` with setup HTML (if not cloned)

---

### `GET /bootstrap`

Validates token and sets authentication cookie.

**Query Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `token` | Yes | Per-session auth token |

**Response:**

- `200 OK` — HTML page that sets cookie and redirects
- `400 Bad Request` — Missing token
- `403 Forbidden` — Invalid token

---

### `POST /setup`

Clones Git repository and saves configuration.

**Form Data:**
| Field | Required | Description |
|-------|----------|-------------|
| `git_url` | Yes | Git repository URL |

**Response:**

- `302 Redirect` to `/` (success)
- `400 Bad Request` — Missing URL
- `500 Internal Server Error` — Clone failed

---

### `GET /web/*`

Serves static files from cloned repository.

**Response:**

- File content with appropriate MIME type
- `404 Not Found` if file doesn't exist

---

### `GET /tbl.js`

JavaScript SDK for API calls.

**Response:**

```javascript
// Provides window.tblApi object
tblApi.ping(); // Health check
tblApi.request(path, opts); // Generic API call
```

---

### `GET /api/v1/ping`

Authenticated health check.

**Headers:**

- `Cookie: tbl_token=...` (required)
- `Authorization: Basic ...` (if configured)

**Response:**

```json
{ "status": "ok" }
```

**Errors:**

- `401 Unauthorized` — Missing/invalid auth

---

### `POST /api/v1/shutdown`

Triggers graceful server shutdown. Used by `tbl --stop`.

**Headers:**

- `Cookie: tbl_token=...` (required)
- `Authorization: Basic ...` (if configured)

**Response:**

```json
{ "status": "shutting_down" }
```

**Errors:**

- `401 Unauthorized` — Missing/invalid auth

## JavaScript SDK

Include in your web UI:

```html
<script src="/tbl.js"></script>
```

### API

```javascript
// Health check
const result = await tblApi.ping();
// { status: "ok" }

// Generic request
const data = await tblApi.request("/some-endpoint", {
  method: "POST",
  body: JSON.stringify({ key: "value" }),
});
```

### Features

- Automatically includes credentials (`credentials: 'include'`)
- Sets `Content-Type: application/json`
- Parses JSON responses automatically
- Throws on non-2xx responses

## Error Responses

| Status                      | Meaning                           |
| --------------------------- | --------------------------------- |
| `400 Bad Request`           | Missing required parameter        |
| `401 Unauthorized`          | Missing or invalid authentication |
| `403 Forbidden`             | Invalid token                     |
| `404 Not Found`             | Resource not found                |
| `500 Internal Server Error` | Server-side error                 |

## CORS

tbl is designed for localhost use. CORS is not configured as all requests originate from the same origin.

## TLS

When TLS is enabled (`--tls-cert` and `--tls-key`):

- All endpoints served over HTTPS
- Browser will need to trust the certificate
- Self-signed certificates work for local development
