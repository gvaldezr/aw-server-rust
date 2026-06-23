# Integration Test Instructions - WebUI Customization

**Unit**: Unit 1 - WebUI Visual Customization  
**Date**: 2026-05-19  
**Purpose**: Verify complete Docker Compose stack runs with Anáhuac Mayab customizations

---

## Integration Test Overview

These tests verify that all three services (PostgreSQL, aw-server, aw-webui) work together correctly with the customized WebUI.

**Test Scope**:
1. Docker Compose stack startup
2. Service health checks
3. Network connectivity between services
4. WebUI accessibility
5. Backend API accessibility
6. Logo and branding load correctly

---

## Prerequisites

- ✅ All builds completed successfully (see build-instructions.md)
- ✅ Docker Compose installed
- ✅ Ports available: 5600 (aw-server), 5666 (aw-webui)

---

## Test 1: Docker Compose Stack Startup

### Steps

#### 1. Navigate to Workspace Root
```bash
cd /Users/guillermo.valdez/Documents/dti-timetracker-apps/aw-rust/aw-server-rust
```

#### 2. Start All Services (Detached)
```bash
docker compose up -d
```

**Expected Output**:
```
[+] Running 3/3
 ✔ Container aw-server-rust-postgresql-1   Started   X.Xs
 ✔ Container aw-server-rust-aw-server-1    Started   X.Xs
 ✔ Container aw-server-rust-aw-webui-1     Started   X.Xs
```

**Startup Time**: ~10-30 seconds

#### 3. Verify All Containers Running
```bash
docker compose ps
```

**Expected Output**:
```
NAME                            STATUS          PORTS
aw-server-rust-postgresql-1     Up X minutes    5432/tcp
aw-server-rust-aw-server-1      Up X minutes    0.0.0.0:5600->5600/tcp
aw-server-rust-aw-webui-1       Up X minutes    0.0.0.0:5666->80/tcp
```

**Critical**: All services must show `Up` status (not `Restarting` or `Exited`)

#### 4. Check Container Health Status
```bash
docker compose ps --format "table {{.Name}}\t{{.Status}}\t{{.Health}}"
```

**Expected Output**:
```
NAME                            STATUS              HEALTH
aw-server-rust-postgresql-1     Up X minutes        healthy
aw-server-rust-aw-server-1      Up X minutes        healthy
aw-server-rust-aw-webui-1       Up X minutes        healthy
```

**Wait Time**: Health checks may take 30-60 seconds to show "healthy"

### Success Criteria

✅ **Stack startup succeeds** when:
- [x] All 3 containers start without errors
- [x] All containers show `Up` status
- [x] All health checks show `healthy` within 2 minutes
- [x] No restart loops (check `docker compose ps` multiple times)

### Troubleshooting

**Issue**: Container shows `Exited (1)` immediately after start
- **Solution**: Check logs with `docker compose logs [service-name]`

**Issue**: Container stuck in `Restarting` loop
- **Solution**: Health check failing - verify service configuration

**Issue**: Port conflict (address already in use)
- **Solution**: Stop other services using ports 5600/5666, or change ports in `docker-compose.yml`

---

## Test 2: Service Health Checks

### Steps

#### 1. Check PostgreSQL Health
```bash
docker compose exec postgresql pg_isready -U aw_user
```

**Expected Output**:
```
/var/run/postgresql:5432 - accepting connections
```

#### 2. Check aw-server API Health
```bash
curl -s http://localhost:5600/api/0/info | jq .
```

**Expected Output** (JSON):
```json
{
  "hostname": "xxxxxx",
  "version": "vX.X.X",
  "testing": false,
  "device_id": "xxxxxx"
}
```

**Note**: If `jq` not installed, omit `| jq .` (output will be compact JSON)

#### 3. Check WebUI HTTP Response
```bash
curl -I http://localhost:5666/
```

**Expected Output**:
```
HTTP/1.1 200 OK
Server: nginx/1.25.x
Content-Type: text/html
...
```

#### 4. Verify WebUI Serves Logo
```bash
curl -I http://localhost:5666/logo.png
```

**Expected Output**:
```
HTTP/1.1 200 OK
Server: nginx/1.25.x
Content-Type: image/png
Content-Length: 22XXX
...
```

**Critical**: Content-Length should be ~22000-23000 bytes (22.5 KB logo)

#### 5. Verify Inter Font Import in WebUI HTML
```bash
curl -s http://localhost:5666/ | grep -o "fonts.googleapis.com/css2?family=Inter"
```

**Expected Output**:
```
fonts.googleapis.com/css2?family=Inter
```

### Success Criteria

✅ **Health checks pass** when:
- [x] PostgreSQL accepts connections
- [x] aw-server API returns valid JSON with version info
- [x] WebUI returns HTTP 200 for index page
- [x] Logo PNG accessible at `/logo.png` (~22.5 KB)
- [x] Inter font import present in HTML

---

## Test 3: Network Connectivity

### Steps

#### 1. Verify aw-server Can Connect to PostgreSQL
```bash
docker compose logs aw-server | grep -i "database\|postgres" | tail -5
```

**Expected Output** (should see successful connection):
```
...INFO aw_server: Using PostgreSQL database at localhost:5432/activitywatch
...INFO aw_datastore: Connected to PostgreSQL successfully
```

**No errors** like "connection refused" or "authentication failed"

#### 2. Verify WebUI Can Reach aw-server API (from host)
```bash
curl -s http://localhost:5666/ | grep -o 'src="/logo.png"'
```

**Expected Output**:
```
src="/logo.png"
```

This confirms WebUI HTML references the logo correctly.

### Success Criteria

✅ **Network connectivity works** when:
- [x] aw-server successfully connects to PostgreSQL
- [x] WebUI HTML loads without proxy errors
- [x] Host can reach both aw-server (5600) and WebUI (5666)

---

## Test 4: Container Logs Verification

### Steps

#### 1. Check PostgreSQL Logs (No Errors)
```bash
docker compose logs postgresql --tail=20
```

**Expected**: 
- No `ERROR` or `FATAL` messages
- Should see `database system is ready to accept connections`

#### 2. Check aw-server Logs (No Errors)
```bash
docker compose logs aw-server --tail=20
```

**Expected**:
- No Rust `panic!` or `error:` messages
- Should see `Rocket has launched from http://0.0.0.0:5600`
- May see INFO/WARN logs (acceptable)

#### 3. Check WebUI Logs (No Errors)
```bash
docker compose logs aw-webui --tail=20
```

**Expected**:
- Nginx access logs (GET requests)
- No 500 errors or permission denied messages

### Success Criteria

✅ **Logs are clean** when:
- [x] No FATAL or ERROR messages in PostgreSQL
- [x] No panic or compilation errors in aw-server
- [x] No 500 errors in WebUI nginx logs
- [x] All services report successful startup

---

## Test 5: End-to-End API Flow

### Steps

#### 1. Create Test Bucket via API
```bash
curl -X POST http://localhost:5600/api/0/buckets/test-bucket-anahuac \
  -H "Content-Type: application/json" \
  -d '{
    "type": "test",
    "client": "integration-test",
    "hostname": "test-host"
  }'
```

**Expected Output**:
```json
{
  "id": "test-bucket-anahuac",
  "type": "test",
  "client": "integration-test",
  "hostname": "test-host",
  "created": "2026-05-19T..."
}
```

#### 2. Verify Bucket Created in Database
```bash
docker compose exec postgresql psql -U aw_user -d activitywatch -c "SELECT name, type FROM buckets WHERE name='test-bucket-anahuac';"
```

**Expected Output**:
```
         name          | type
-----------------------+------
 test-bucket-anahuac   | test
(1 row)
```

#### 3. Insert Test Event
```bash
curl -X POST http://localhost:5600/api/0/buckets/test-bucket-anahuac/events \
  -H "Content-Type: application/json" \
  -d '[{
    "timestamp": "2026-05-19T12:00:00Z",
    "duration": 60,
    "data": {"app": "TestApp", "title": "Anáhuac Mayab Test"}
  }]'
```

**Expected**: HTTP 200 response

#### 4. Retrieve Events via API
```bash
curl -s http://localhost:5600/api/0/buckets/test-bucket-anahuac/events | jq '.[0].data'
```

**Expected Output**:
```json
{
  "app": "TestApp",
  "title": "Anáhuac Mayab Test"
}
```

#### 5. Cleanup Test Data
```bash
curl -X DELETE http://localhost:5600/api/0/buckets/test-bucket-anahuac
```

### Success Criteria

✅ **End-to-end flow works** when:
- [x] Bucket creation succeeds
- [x] Bucket visible in PostgreSQL
- [x] Event insertion succeeds
- [x] Event retrieval returns correct data
- [x] Bucket deletion succeeds

---

## Integration Test Checklist

### Startup Tests
- [ ] `docker compose up -d` starts all 3 services
- [ ] All containers show `Up` status
- [ ] All health checks show `healthy`
- [ ] No restart loops observed

### Health Check Tests
- [ ] PostgreSQL `pg_isready` succeeds
- [ ] aw-server `/api/0/info` returns valid JSON
- [ ] WebUI returns HTTP 200 for `/`
- [ ] Logo accessible at `/logo.png` (~22.5 KB)
- [ ] Inter font import in HTML

### Network Tests
- [ ] aw-server connects to PostgreSQL successfully
- [ ] Host can reach aw-server on port 5600
- [ ] Host can reach WebUI on port 5666

### Log Tests
- [ ] No FATAL/ERROR in PostgreSQL logs
- [ ] No panic/error in aw-server logs
- [ ] No 500 errors in WebUI logs

### API Tests
- [ ] Bucket creation succeeds
- [ ] Bucket visible in database
- [ ] Event insertion succeeds
- [ ] Event retrieval succeeds
- [ ] API returns correct data

---

## Monitoring Commands

### Real-Time Logs (All Services)
```bash
docker compose logs -f
```

Press `Ctrl+C` to exit.

### Resource Usage
```bash
docker stats
```

**Expected**:
- PostgreSQL: ~50-200 MB RAM
- aw-server: ~20-100 MB RAM
- aw-webui: ~5-20 MB RAM (nginx)

### Container Restart Count
```bash
docker compose ps --format "table {{.Name}}\t{{.Status}}"
```

Look for "(healthy)" vs "(health: starting)" vs "Restarting"

---

## Cleanup After Tests

### Stop All Services
```bash
docker compose down
```

### Stop and Remove Volumes (Full Cleanup)
```bash
docker compose down -v
```

**Warning**: This deletes all data in PostgreSQL.

---

## Next Steps

After successful integration tests:
1. **Proceed to visual-verification-instructions.md** - Open WebUI in browser and verify Anáhuac branding
2. **Proceed to functional-test-instructions.md** - Test navigation, buttons, and features

---

## Troubleshooting

### Container Won't Start

**Check logs**:
```bash
docker compose logs [service-name]
```

**Common Issues**:
- Port already in use → Change port or stop conflicting service
- Volume permission errors → Check Docker volume permissions
- Health check timeout → Increase health check intervals in docker-compose.yml

### Services Start But Not Healthy

**PostgreSQL**:
```bash
docker compose exec postgresql psql -U aw_user -d activitywatch -c "SELECT 1;"
```
If this fails, check database initialization.

**aw-server**:
```bash
curl http://localhost:5600/api/0/info
```
If this fails, check aw-server can connect to PostgreSQL.

### WebUI Shows Blank Page

1. Check nginx is serving files:
   ```bash
   docker compose exec aw-webui ls -la /usr/share/nginx/html/
   ```
2. Verify `index.html` and `logo.png` present
3. Check browser console for JavaScript errors

---

**Integration Test Instructions Complete**  
**Status**: Ready for Visual Verification
