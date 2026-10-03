# MVSep API Endpoint Reference

> Source: https://mvsep.com/zh/full_api
> API information captured: 2026-06-27
> English translation: 2026-10-03

This reference describes the captured API information. See the upstream documentation for subsequent changes.

## Algorithm discovery

**GET** `https://mvsep.com/api/app/algorithms`

Returns the available separation algorithms and their parameter options. Fetch these definitions dynamically rather than hard-coding models or option values.

- `render_id`: separation type ID (`sep_type`).
- `name`: algorithm name.
- `algorithm_fields`: additional parameters (`add_opt1`, `add_opt2`, `add_opt3`) and their options.
- `price_coefficient`: credit cost multiplier.
- `orientation`: user permission requirement.

| Query parameter | Type | Description |
|---|---|---|
| scopes | string | Optional model scope: `single_upload` (default), `no_upload`, or `matchering_upload` (audio matching). |

Illustrative response structure (option values depend on the algorithm):

```json
[
  {
    "render_id": 26,
    "name": "Ensemble (vocals, instrum)",
    "algorithm_group": { "name": "Vocal/instrumental separation" },
    "algorithm_fields": [
      {
        "name": "add_opt1",
        "text": "Output files",
        "options": { "0": "Standard set", "1": "Include intermediate results" },
        "default_key": "0"
      },
      {
        "name": "add_opt2",
        "text": "Model Type",
        "options": { "7": "Example model" },
        "default_key": "7"
      }
    ],
    "price_coefficient": 1.0,
    "orientation": 0
  }
]
```

## Action endpoints

### 1. Create a separation task

**POST** `https://mvsep.com/api/separation/create`

Uploads an audio file and creates a separation task.

| Parameter | Type | Required | Description |
|---|---|---|---|
| audiofile | binary | Yes | Audio file (MP3, WAV, FLAC, OGG, WEBM, MP4A, AAC). Captured limits: 50 MB for free users, 200 MB for paid users. |
| api_token | string | Yes | User API token. |
| sep_type | integer | Yes | Separation type ID from `/api/app/algorithms`. |
| add_opt1 | integer/string | No | Additional option 1; depends on the algorithm. |
| add_opt2 | integer/string | No | Additional option 2. |
| output_format | integer | No | `0` = MP3, `1` = WAV (16-bit), `2` = WAV (24-bit), `3` = WAV (32-bit float), `4` = WAV (32-bit), `5` = FLAC. |
| is_demo | integer | No | Demo mode: `0` = disabled, `1` = enabled. |

```bash
curl --location --request POST 'https://mvsep.com/api/separation/create' \
  --form 'audiofile=@"/path/to/file.mp3"' \
  --form 'api_token="<YOUR_API_TOKEN>"' \
  --form 'sep_type="26"' \
  --form 'add_opt1="0"' \
  --form 'add_opt2="7"' \
  --form 'output_format="1"' \
  --form 'is_demo="0"'
```

Response:

```json
{
  "success": true,
  "data": {
    "link": "https://mvsep.com/api/separation/get?hash=xxx",
    "hash": "20230327071601-xxx.mp3"
  }
}
```

Errors: `400` for invalid or missing parameters; `401` for an invalid API token.

### 2. Cancel a separation task

**POST** `https://mvsep.com/api/separation/cancel`

Cancels a task that has not started processing and refunds its credits.

| Parameter | Type | Required | Description |
|---|---|---|---|
| api_token | string | Yes | User API token. |
| hash | string | Yes | Task hash returned when the task was created. |

```bash
curl --location --request POST 'https://mvsep.com/api/separation/cancel' \
  --form 'api_token="<YOUR_API_TOKEN>"' \
  --form 'hash="<TASK_HASH>"'
```

### 3. Log in

**POST** `https://mvsep.com/api/app/login`

Authenticates a user and returns an API token.

| Parameter | Type | Required | Description |
|---|---|---|---|
| email | string | Yes | User email address. |
| password | string | Yes | User password. |

```bash
curl --location --request POST 'https://mvsep.com/api/app/login' \
  --form 'email="user@example.com"' \
  --form 'password="your_password"'
```

Response:

```json
{
  "success": true,
  "data": {
    "name": "username",
    "email": "user@example.com",
    "api_token": "xxxxx",
    "premium_minutes": 100,
    "premium_enabled": 1,
    "long_filenames_enabled": 0
  }
}
```

Save `api_token` for subsequent authenticated calls. `premium_minutes` is the remaining credit balance, `premium_enabled` controls credit usage, and `long_filenames_enabled` controls the output filename format.

### 4. Register a user

**POST** `https://mvsep.com/api/app/register`

| Parameter | Type | Required | Description |
|---|---|---|---|
| name | string | Yes | Username. |
| email | string | Yes | Email address. |
| password | string | Yes | Password. |
| password_confirmation | string | Yes | Password confirmation. |

```bash
curl --location --request POST 'https://mvsep.com/api/app/register' \
  --form 'name="username"' \
  --form 'email="user@example.com"' \
  --form 'password="SecurePass123!"' \
  --form 'password_confirmation="SecurePass123!"'
```

### 5. Enable premium processing

**POST** `https://mvsep.com/api/app/enable_premium`

Allows tasks to consume credits for faster processing. Requires the string parameter `api_token`.

### 6. Disable premium processing

**POST** `https://mvsep.com/api/app/disable_premium`

Prevents tasks from consuming credits. Requires the string parameter `api_token`.

### 7. Enable long filenames

**POST** `https://mvsep.com/api/app/enable_long_filenames`

Includes additional information such as the algorithm name and options in output filenames. Requires the string parameter `api_token`.

### 8. Disable long filenames

**POST** `https://mvsep.com/api/app/disable_long_filenames`

Uses shorter output filenames. Requires the string parameter `api_token`.

### 9. Add a quality checker entry

**POST** `https://mvsep.com/api/quality_checker/add`

Submits an algorithm to the quality checker leaderboard.

| Parameter | Type | Required | Description |
|---|---|---|---|
| api_token | string | Yes | API token. |
| zipfile | binary | Yes | ZIP file to process. |
| algo_name | string | Yes | Algorithm name. |
| main_text | string | Yes | Algorithm description. |
| dataset_type | string | No | Dataset type (`0`–`12`). |
| ensemble | integer | No | Ensemble model: `0` = no, `1` = yes. |
| password | string | Yes | Password used to delete the entry. |

### 10. Delete a quality checker entry

**POST** `https://mvsep.com/api/quality_checker/delete`

| Parameter | Type | Required | Description |
|---|---|---|---|
| id | integer | Yes | Entry ID. |
| password | string | Yes | Deletion password. |

## Query endpoints

### 1. Get separation results

**GET** `https://mvsep.com/api/separation/get`

Returns the task status and download links.

| Parameter | Type | Required | Description |
|---|---|---|---|
| hash | string | Yes | Task hash. |
| mirror | integer | No | Mirror download: `0` = disabled, `1` = enabled (requires `api_token` and one credit). |
| api_token | string | Conditional | Required when `mirror=1`. |

```bash
curl --location --request GET 'https://mvsep.com/api/separation/get?hash=20230327071601-xxx.mp3'
```

| Status | Meaning |
|---|---|
| `not_found` | Invalid task. |
| `waiting` | Queued. |
| `processing` | Processing. |
| `done` | Complete; download links are available. |
| `failed` | Processing failed. |
| `distributing` | Distributing large files. |
| `merging` | Merging results. |

Completed response:

```json
{
  "success": true,
  "status": "done",
  "data": {
    "algorithm": "BS Roformer",
    "algorithm_description": "...",
    "output_format": "wav",
    "input_file": { "link": "...", "size": 10240000 },
    "files": [
      { "name": "vocals.wav", "link": "...", "size": 5000000 },
      { "name": "instrumental.wav", "link": "...", "size": 5200000 }
    ],
    "date": "2023-03-27 07:20:00"
  }
}
```

Queued response:

```json
{
  "success": true,
  "status": "waiting",
  "data": {
    "queue_count": 15,
    "current_order": 3,
    "message": "You are number 3 in the queue"
  }
}
```

### 2. Get remote task results

**GET** `https://mvsep.com/api/separation/get-remote`

Returns the status of a remotely submitted task. Requires the string parameter `hash` (the remote task hash). The response has the same structure as above, but returns a new `hash` when `status` is `done`; use that hash to retrieve the actual results.

### 3. Get separation algorithms

**GET** `https://mvsep.com/api/app/algorithms`

Returns the available algorithms and parameter definitions; see [Algorithm discovery](#algorithm-discovery). Call this endpoint before building algorithm selection controls so that the UI uses the available algorithms and their valid options.

### 4. Get user information

**GET** `https://mvsep.com/api/app/user`

Returns the current user and account status. Requires the string parameter `api_token`.

```json
{
  "success": true,
  "data": {
    "name": "username",
    "email": "user@example.com",
    "api_token": "xxxxx",
    "premium_minutes": 100,
    "premium_enabled": 1,
    "long_filenames_enabled": 0,
    "current_queue": null
  }
}
```

`current_queue` describes the task currently being processed.

### 5. Get separation history

**GET** `https://mvsep.com/api/app/separation_history`

| Parameter | Type | Required | Description |
|---|---|---|---|
| api_token | string | Yes | User API token. |
| start | integer | No | Offset (default `0`, newest first). |
| limit | integer | No | Result count (default `10`, maximum `20`). |

### 6. Get site queue status

**GET** `https://mvsep.com/api/app/queue`

Returns the server load and queue status. An optional string parameter `api_token` provides queue information for the user's plan.

```json
{
  "queue": {
    "in_process": 25,
    "premium": 5,
    "registered": 150,
    "unregistered": 800
  },
  "plan": {
    "plan": "free",
    "queue": 800
  }
}
```

### 7. Get news

**GET** `https://mvsep.com/api/app/news`

Returns MVSEP news.

| Parameter | Type | Required | Description |
|---|---|---|---|
| lang | string | No | Language code, such as `en`, `ru`, or `zh`. |
| start | integer | No | Offset. |
| limit | integer | No | Result count (default `10`, maximum `20`). |

### 8. Get demo separations

**GET** `https://mvsep.com/api/app/demo`

Returns official demo samples.

| Parameter | Type | Required | Description |
|---|---|---|---|
| start | integer | No | Offset. |
| limit | integer | No | Result count. |
| algorithm_id | integer | No | Filter by algorithm ID. |
| options[FIELD] | mixed | No | Filter by an option; first retrieve field names from the algorithms endpoint. |
| additional_options | string | No | Filter using raw option JSON (not recommended). |

```bash
# Get all demos.
curl 'https://mvsep.com/api/app/demo?start=0&limit=10'

# Filter demos for a specific algorithm.
curl 'https://mvsep.com/api/app/demo?algorithm_id=26&options[add_opt2]=7&start=0&limit=10'
```

### 9. Get the quality checker queue

**GET** `https://mvsep.com/api/quality_checker/queue`

| Parameter | Type | Required | Description |
|---|---|---|---|
| start | integer | No | Offset. |
| limit | integer | No | Result count. |
| algorithm_id | integer | No | Filter by algorithm ID. |
| options[FIELD] | mixed | No | Filter by an option. |

### 10. Get the quality checker leaderboard

**GET** `https://mvsep.com/api/quality_checker/leaderboard`

| Parameter | Type | Required | Description |
|---|---|---|---|
| dataset_type | string | No | Dataset type (`0`–`12`): `0` = Synth, `1` = Multi, `2` = Piano, `3` = Lead/Back Vocals, `4` = Guitar, etc. |
| start | integer | No | Offset. |
| limit | integer | No | Result count. |
| algo_name_filter | string | No | Search by algorithm name. |
| sort | string | No | Sort field from the response's `sortables`. |

### 11. Get a quality checker entry

**GET** `https://mvsep.com/api/quality_checker/entry`

Returns details for a single entry. Requires the integer parameter `id`.

## Typical workflows

### Basic audio separation

1. Log in with `POST /api/app/login` to obtain `api_token`.
2. Retrieve algorithms with `GET /api/app/algorithms` to build the selection UI.
3. Upload audio and the selected options with `POST /api/separation/create`; save the returned `hash`.
4. Poll `GET /api/separation/get?hash=xxx` until `status` is `done`.
5. Download the separated audio from `data.files[].link`.

### Build an algorithm selector dynamically

1. Retrieve the full list with `GET /api/app/algorithms`.
2. Extract each algorithm's `render_id` (`sep_type`), `name`, `price_coefficient`, and `orientation`.
3. Read `algorithm_fields[]`: `name` (parameter key such as `add_opt1`), `text` (display label), `options` (available values), and `default_key`.
4. Render the corresponding controls, such as dropdowns.
5. Submit the selected values to `POST /api/separation/create`.

### Remote task processing

For audio stored on the server or referenced by URL:

1. Submit `POST /api/separation/create` in the appropriate special mode to obtain a remote hash. The captured reference does not specify that mode's parameters.
2. Poll `GET /api/separation/get-remote?hash=xxx` until `status` is `done` and a new hash is returned.
3. Use `GET /api/separation/get?hash=<NEW_HASH>` to retrieve the actual results.

## Endpoint summary

### Actions (10 endpoints)

| Method | Endpoint | Purpose | Authentication |
|---|---|---|---|
| POST | `/api/separation/create` | Create a separation task. | API token |
| POST | `/api/separation/cancel` | Cancel a task. | API token |
| POST | `/api/app/login` | Log in. | Email/password |
| POST | `/api/app/register` | Register a user. | No token |
| POST | `/api/app/enable_premium` | Enable credit usage. | API token |
| POST | `/api/app/disable_premium` | Disable credit usage. | API token |
| POST | `/api/app/enable_long_filenames` | Enable long filenames. | API token |
| POST | `/api/app/disable_long_filenames` | Disable long filenames. | API token |
| POST | `/api/quality_checker/add` | Submit a quality checker entry. | API token |
| POST | `/api/quality_checker/delete` | Delete an entry. | Deletion password |

### Queries (11 endpoints)

| Method | Endpoint | Purpose | Authentication |
|---|---|---|---|
| GET | `/api/separation/get` | Get task results. | No token unless using a mirror |
| GET | `/api/separation/get-remote` | Get remote task results. | No token |
| GET | `/api/app/algorithms` | Get algorithms and options. | No token |
| GET | `/api/app/user` | Get user information. | API token |
| GET | `/api/app/separation_history` | Get task history. | API token |
| GET | `/api/app/queue` | Get queue status. | Optional API token |
| GET | `/api/app/news` | Get news. | No token |
| GET | `/api/app/demo` | Get demos. | No token |
| GET | `/api/quality_checker/queue` | Get the quality checker queue. | No token |
| GET | `/api/quality_checker/leaderboard` | Get the leaderboard. | No token |
| GET | `/api/quality_checker/entry` | Get entry details. | No token |

## HTTP status codes

| Code | Meaning | Typical cause |
|---|---|---|
| `200` | Success | Request completed successfully. |
| `400` | Bad request | Invalid/missing parameters, form validation failure, or incorrect credentials. |
| `401` | Unauthorized | Invalid or unknown API token. |

## Usage recommendations from the captured reference

1. Cache `/api/app/algorithms` results, which change infrequently, rather than fetching them for every request.
2. Poll task status every 5–10 seconds.
3. Continue polling for `waiting` and `processing`; investigate `failed` before retrying.
4. Check `/api/app/user` for the credit balance before creating a task that consumes credits.
5. Observe upload size limits (captured values: 50 MB free, 200 MB paid).
6. WAV provides higher quality with larger files; MP3 is suitable for quick previews.

This document was originally assembled from HTML retrieved with curl. Examples illustrate response structure; labels and messages may vary by server language.
