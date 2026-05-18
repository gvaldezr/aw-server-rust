# Business Overview - ActivityWatch Server

## Business Context

**ActivityWatch** is an open-source time tracking application that monitors computer activity automatically. The system tracks user activity through various watchers (applications, web browsing, mouse/keyboard activity) and stores this data in a centralized server for analysis and reporting.

## Core Business Transactions

1. **Event Recording**: Watchers capture activity events (application usage, web browsing, idle time) and send them to the server
2. **Bucket Management**: Create and manage data buckets (logical containers for events from specific watchers)
3. **Event Query**: Retrieve and analyze events across time periods with filtering capabilities
4. **Data Transformation**: Process raw events into meaningful insights (heartbeat merging, classification, period filtering)
5. **Export/Import**: Export activity data for backup/migration and import from legacy systems

## Business Dictionary

- **Bucket**: A logical container for events from a specific watcher, identified by bucket_id
- **Event**: A single activity record with timestamp, duration, and data payload
- **Watcher**: A client application that captures activity and sends events to the server
- **Heartbeat**: Continuous activity events indicating user presence (merged into single events)
- **Activity Period**: A time interval during which the user was active

## System Scope

The server is the **backend infrastructure** that:
- Provides RESTful API for watchers to submit events
- Manages persistent storage of activity data
- Exposes query and analysis capabilities
- Serves the web UI (aw-webui) for visualization
- Handles data transformations and advanced queries
