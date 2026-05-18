# API Documentation

## REST API Overview

The ActivityWatch Server exposes a RESTful API for managing buckets, events, and queries. Base URL: `http://localhost:5600/api/0/`

## Bucket Management APIs

### Create Bucket
- **Method**: `PUT`
- **Path**: `/api/0/buckets/{bucket_id}`
- **Purpose**: Create a new bucket for a watcher
- **Request Body**: Bucket configuration with type, client, hostname
- **Response**: Created bucket object

### Get Bucket
- **Method**: `GET`
- **Path**: `/api/0/buckets/{bucket_id}`
- **Purpose**: Retrieve bucket metadata
- **Response**: Bucket object with configuration

### Delete Bucket
- **Method**: `DELETE`
- **Path**: `/api/0/buckets/{bucket_id}`
- **Purpose**: Remove a bucket and all associated events
- **Response**: Success confirmation

### List Buckets
- **Method**: `GET`
- **Path**: `/api/0/buckets`
- **Purpose**: Get all buckets
- **Response**: Array of bucket objects

## Event Management APIs

### Submit Event
- **Method**: `POST`
- **Path**: `/api/0/buckets/{bucket_id}/events`
- **Purpose**: Add an event to a bucket
- **Request Body**: Event object with timestamp, duration, and data
- **Response**: Created event with ID

### Get Events
- **Method**: `GET`
- **Path**: `/api/0/buckets/{bucket_id}/events`
- **Query Parameters**:
  - `start`: Start timestamp (ISO 8601)
  - `end`: End timestamp (ISO 8601)
  - `limit`: Maximum number of events
- **Purpose**: Retrieve events within time range
- **Response**: Array of events

### Delete Event
- **Method**: `DELETE`
- **Path**: `/api/0/buckets/{bucket_id}/events/{event_id}`
- **Purpose**: Remove a specific event
- **Response**: Success confirmation

## Advanced Query API

### Execute Query
- **Method**: `POST`
- **Path**: `/api/0/query`
- **Purpose**: Execute advanced query against activity data
- **Request Body**:
  ```json
  {
    "query": "find_bucket(type='window') | merge_events_by_keys(\"app\") | limit(10)",
    "timeperiods": [["2024-01-01T00:00:00Z", "2024-01-02T00:00:00Z"]]
  }
  ```
- **Response**: Query results as array of events

## Data Import/Export APIs

### Export Data
- **Method**: `GET`
- **Path**: `/api/0/export`
- **Query Parameters**:
  - `format`: Export format (json, csv)
  - `start`: Start timestamp
  - `end`: End timestamp
- **Purpose**: Export activity data
- **Response**: Data in requested format

### Import Data
- **Method**: `POST`
- **Path**: `/api/0/import`
- **Purpose**: Import activity data (bulk insert)
- **Request Body**: Bucket and event data
- **Response**: Import status/summary

## Settings/Configuration APIs

### Get Settings
- **Method**: `GET`
- **Path**: `/api/0/settings`
- **Purpose**: Retrieve server settings
- **Response**: Current settings object

### Update Settings
- **Method**: `POST`
- **Path**: `/api/0/settings`
- **Purpose**: Update server settings
- **Request Body**: Updated settings
- **Response**: Updated settings object

## Server Info API

### Get Server Info
- **Method**: `GET`
- **Path**: `/api/0/info`
- **Purpose**: Get server metadata and version info
- **Response**: Server information including version, hostname, device_id

## Authentication

- **Method**: Bearer token via Authorization header
- **Configuration**: Optional API key in `config.toml` under `[auth].api_key`
- **Endpoints Requiring Auth**: All `/api/*` endpoints except `/api/0/info`
- **Example**: `Authorization: Bearer <api_key>`

## CORS Configuration

**Default Allowed Origins:**
- `http://127.0.0.1:<port>`
- `http://localhost:<port>`
- `chrome-extension://nglaklhklhcoonedhgnpgddginnjdadi` (official Chrome extension)
- `moz-extension://.*` (all Firefox extensions)

**Additional Origins** can be configured in `config.toml`

## Internal APIs

### Database Query Interface
- Location: `aw-datastore` module
- Method: MPSC channel-based requests
- Pattern: Request → Database Worker → Response

### Query Engine Interface
- Location: `aw-query` module
- Operations: Parse query string → Execute against events → Return results
