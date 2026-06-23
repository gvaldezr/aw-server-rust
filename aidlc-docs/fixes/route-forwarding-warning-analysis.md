# Fix 7: Route Forwarding Warning Analysis

**Date**: 2026-05-20  
**Component**: aw-server (Rocket web framework)  
**Severity**: Informational (WARN level)  
**Status**: Accepted as designed behavior

---

## Issue Description

Server logs show a warning when accessing the `/events/count` endpoint:

```
[WARN][aw_server::endpoints::bucket::_]: Parameter guard `event_id: i64` is forwarding: "count".
```

**Example request**:
```bash
curl http://localhost:5600/api/0/buckets/aw-watcher-window_SUPC03/events/count
# Returns: 134 ✅ (endpoint works correctly)
```

---

## Root Cause Analysis

### Rocket Route Matching Behavior

Rocket evaluates routes in the order they are **registered** in `mod.rs`, not the order they are defined in `bucket.rs`.

**Conflicting routes**:
```rust
// Generic route (catches all segments, including "count")
#[get("/<bucket_id>/events/<event_id>?<_unused..>")]
pub async fn bucket_events_get_single(
    bucket_id: &str,
    event_id: i64,  // Type guard: must be a valid i64
    ...
)

// Specific route (literal "count" segment)
#[get("/<bucket_id>/events/count")]
pub async fn bucket_event_count(...)
```

**What happens**:
1. Request: `GET /buckets/aw-watcher-window_SUPC03/events/count`
2. Rocket tries first route: `/<bucket_id>/events/<event_id>`
3. Type guard `event_id: i64` fails because "count" is not a valid integer
4. Rocket **forwards** to next matching route ⚠️ (this generates the warning)
5. Second route matches: `/<bucket_id>/events/count` ✅
6. Endpoint executes successfully and returns result

### Investigation Attempts

**Attempt 1: Route Definition Order**
- Reordered `bucket_event_count` before `bucket_events_get_single` in `bucket.rs`
- Result: ❌ No effect (routes evaluated by registration order, not definition order)

**Attempt 2: Route Registration Order**
- Reordered routes in `mod.rs` to register `bucket_event_count` first
- Result: ❌ Warning persists (both routes still match the path pattern)

**Attempt 3: Explicit Route Ranking**
- Added `rank = 1` to specific route and `rank = 2` to generic route
- Result: ❌ Compilation error with negative ranks, warning persists with positive ranks

---

## Technical Explanation

This warning is **by design** in Rocket's routing system:

1. **Route Collisions**: Multiple routes can match the same URL path
2. **Type Guards**: Parameter guards (like `i64`) act as validators
3. **Forwarding**: When a guard fails, Rocket forwards to the next matching route
4. **Warning Level**: WARN indicates forwarding occurred, not an error

From Rocket documentation:
> When a route's guard fails, Rocket forwards the request to the next matching route. This is called route forwarding.

**Why the warning exists**:
- Informs developers that forwarding is occurring (performance consideration)
- Helps identify potential route ambiguity
- Does NOT indicate a functional problem

---

## Code State

### bucket.rs (lines 133-160)
```rust
// IMPORTANT: Specific routes must come BEFORE generic routes to avoid forwarding warnings
// Place /count before /<event_id> to prevent "count" from being parsed as an i64
#[get("/<bucket_id>/events/count")]
pub async fn bucket_event_count(
    bucket_id: &str,
    state: &State<ServerState>,
) -> Result<Json<u64>, HttpErrorJson> {
    let datastore = {
        let ds = endpoints_get_lock!(state.datastore);
        ds.clone()
    };
    let res = datastore.get_event_count(bucket_id, None, None).await;
    match res {
        Ok(eventcount) => Ok(Json(eventcount as u64)),
        Err(err) => Err(err.into()),
    }
}

// Generic route with lower priority to avoid matching "count" as an integer
// Needs unused parameter, otherwise there'll be a route collision
// See: https://api.rocket.rs/master/rocket/struct.Route.html#resolving-collisions
#[get("/<bucket_id>/events/<event_id>?<_unused..>")]
pub async fn bucket_events_get_single(
    bucket_id: &str,
    event_id: i64,
    _unused: Option<u64>,
    state: &State<ServerState>,
) -> Result<Json<Event>, HttpErrorJson> {
    ...
}
```

### mod.rs (lines 174-177)
```rust
bucket::bucket_event_count,        // MUST be before bucket_events_get_single
bucket::bucket_events_get_single,  // Generic route, goes after specific ones
```

---

## Decision: Accept Warning

**Rationale**:
1. ✅ **Endpoint functions correctly** - returns accurate event count
2. ✅ **Only WARN level** - not ERROR or CRITICAL
3. ✅ **Expected Rocket behavior** - forwarding is a feature, not a bug
4. ✅ **No performance impact** - minimal overhead from one type check
5. ✅ **No functional impact** - users see correct results

**Alternatives considered**:

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| **Accept warning** | Simple, no code changes, works correctly | Warning in logs | ✅ **Selected** |
| Change API to `/events?action=count` | Eliminates collision | Breaking change, requires updating all clients | ❌ Too disruptive |
| Custom type guard | Eliminates warning | Adds complexity without benefit | ❌ Over-engineering |
| Suppress warning | Clean logs | Hides information | ❌ Not transparent |

---

## Verification

**Endpoint functionality**:
```bash
$ curl -s http://localhost:5600/api/0/buckets/aw-watcher-window_SUPC03/events/count
134

$ curl -s http://localhost:5600/api/0/buckets/aw-watcher-afk_SUPC03/events/count
0
```

**Log output**:
```
[2026-05-20 17:54:41][WARN][aw_server::endpoints::bucket::_]: Parameter guard `event_id: i64` is forwarding: "count".
```

**Status**: ✅ Working as designed

---

## Monitoring Recommendations

1. **Filter warning in production logs** if it creates noise:
   ```bash
   docker compose logs aw-server | grep -v "forwarding.*count"
   ```

2. **Monitor for actual routing errors** (HTTP 404, 500):
   ```bash
   docker compose logs aw-server | grep -E "ERROR|404|500"
   ```

3. **Track endpoint performance** to ensure no degradation

---

## Related Issues

- **Issue 1**: Flood warnings (RESOLVED ✅)
- **Issue 2**: Bucket creation errors (RESOLVED ✅)
- **Issue 3-6**: Frontend TypeErrors (RESOLVED ✅)
- **Issue 7**: Route forwarding warning (ACCEPTED ✅)
- **Issue 8**: Window bucket TypeError (PENDING 🔍)

---

## References

- [Rocket Route Collisions](https://api.rocket.rs/master/rocket/struct.Route.html#resolving-collisions)
- [Rocket Request Guards](https://rocket.rs/guide/requests/#request-guards)
- ActivityWatch API: `/api/0/buckets/<bucket_id>/events/count`
