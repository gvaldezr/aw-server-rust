# Build Instructions - WebUI Customization

**Unit**: Unit 1 - WebUI Visual Customization  
**Date**: 2026-05-19  
**Purpose**: Validate Anáhuac Mayab branding implementation builds successfully

---

## Build Overview

This document provides step-by-step instructions to build the customized ActivityWatch WebUI with Anáhuac Mayab branding.

**Build Targets**:
1. **Local npm build** - Development verification
2. **Docker WebUI image** - Production container
3. **Full Docker Compose stack** - Integration verification

---

## Prerequisites

### Required Software
- **Node.js**: 20.x (for local builds)
- **npm**: 9.x or 10.x
- **Docker**: 20.x or later
- **Docker Compose**: 2.x or later

### Verification Commands
```bash
node --version    # Should show v20.x.x
npm --version     # Should show 9.x.x or 10.x.x
docker --version  # Should show 20.x or later
docker compose version  # Should show 2.x or later
```

---

## Build 1: Local npm Build (Development)

**Purpose**: Verify customized WebUI compiles without errors

### Steps

#### 1. Navigate to WebUI Directory
```bash
cd /Users/guillermo.valdez/Documents/dti-timetracker-apps/aw-rust/aw-server-rust/aw-webui
```

#### 2. Install Dependencies
```bash
npm install
```

**Expected Output**:
```
added XXX packages in Xs
```

**Note**: If you see peer dependency warnings, they are expected with Vue 2.7.

#### 3. Run Production Build
```bash
npm run build
```

**Expected Output**:
```
vite v4.x.x building for production...
✓ XXX modules transformed.
dist/index.html                  X.XX kB
dist/assets/index-XXXXXX.js      XXX kB │ gzip: XX kB
dist/assets/index-XXXXXX.css     XX kB  │ gzip: X kB
✓ built in Xs
```

#### 4. Verify Build Artifacts
```bash
ls -lh dist/
```

**Expected Files**:
- `index.html` - Entry point
- `logo.png` - Anáhuac logo (22.5 KB)
- `assets/` - JavaScript and CSS bundles
- `favicon.ico` - Favicon

#### 5. Check Logo Presence
```bash
file dist/logo.png
ls -lh dist/logo.png
```

**Expected Output**:
```
dist/logo.png: PNG image data, 512 x 512, 8-bit/color RGBA, non-interlaced
-rw-r--r--  1 user  staff   22K May 19 01:00 dist/logo.png
```

#### 6. Verify Inter Font Import in Built HTML
```bash
grep -A2 "fonts.googleapis.com" dist/index.html
```

**Expected Output**:
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;700&display=swap" rel="stylesheet">
```

### Success Criteria

✅ **Local npm build succeeds** when:
- [x] `npm install` completes without fatal errors
- [x] `npm run build` completes without errors
- [x] `dist/` directory created with all expected files
- [x] `dist/logo.png` exists and is 22.5 KB PNG
- [x] Inter font import present in `dist/index.html`
- [x] No console errors during build
- [x] Build time < 2 minutes

### Troubleshooting

**Issue**: `npm install` fails with network errors
- **Solution**: Check internet connection, retry with `npm install --verbose`

**Issue**: `npm run build` fails with "Cannot find module"
- **Solution**: Delete `node_modules/` and `package-lock.json`, run `npm install` again

**Issue**: Build succeeds but `logo.png` missing from `dist/`
- **Solution**: Verify `static/logo.png` exists in source, check Vite static asset handling

---

## Build 2: Docker WebUI Image

**Purpose**: Build production-ready Docker image with customized WebUI

### Steps

#### 1. Navigate to Workspace Root
```bash
cd /Users/guillermo.valdez/Documents/dti-timetracker-apps/aw-rust/aw-server-rust
```

#### 2. Verify Dockerfile.webui Uses Local Source
```bash
grep -A2 "COPY aw-webui" Dockerfile.webui
```

**Expected Output**:
```dockerfile
# Copy local customized aw-webui (Anáhuac Mayab branding)
COPY aw-webui /build
```

**Critical**: If you see `git clone` instead, restore from backup:
```bash
# DO NOT RUN unless Dockerfile is wrong
# cp Dockerfile.webui.backup Dockerfile.webui
```

#### 3. Build WebUI Docker Image
```bash
docker compose build aw-webui
```

**Expected Output** (abbreviated):
```
[+] Building 120.5s (12/12) FINISHED
 => [internal] load build definition from Dockerfile.webui
 => [internal] load .dockerignore
 => [internal] load metadata for docker.io/library/node:20-alpine
 => [internal] load metadata for docker.io/library/nginx:1.25-alpine
 => [builder 1/4] FROM docker.io/library/node:20-alpine
 => [internal] load build context
 => [builder 2/4] COPY aw-webui /build
 => [builder 3/4] RUN npm ci
 => [builder 4/4] RUN npm run build
 => [stage-1 2/3] COPY --from=builder /build/dist /usr/share/nginx/html
 => [stage-1 3/3] COPY docker/nginx.conf /etc/nginx/nginx.conf
 => exporting to image
 => => naming to docker.io/library/aw-server-rust-aw-webui
```

**Build Time**: ~60-120 seconds (first build), ~10-30 seconds (cached)

#### 4. Verify Image Created
```bash
docker images | grep aw-webui
```

**Expected Output**:
```
aw-server-rust-aw-webui   latest   XXXXXXXXXXXX   X minutes ago   XX MB
```

**Image Size**: ~50-80 MB (nginx:1.25-alpine + static files)

#### 5. Inspect Image Layers (Optional)
```bash
docker history aw-server-rust-aw-webui:latest --no-trunc | head -10
```

### Success Criteria

✅ **Docker WebUI image build succeeds** when:
- [x] Build completes without errors
- [x] Image tagged as `aw-server-rust-aw-webui:latest`
- [x] Image size reasonable (~50-80 MB)
- [x] Build uses local `aw-webui/` source (not git clone)
- [x] Build time < 3 minutes

### Troubleshooting

**Issue**: Build fails with "COPY failed: no source files"
- **Solution**: Verify `aw-webui/` directory exists with all files, check `.dockerignore` doesn't exclude required files

**Issue**: `npm ci` fails in Docker build
- **Solution**: Check `aw-webui/package-lock.json` exists and is valid

**Issue**: Build uses wrong Dockerfile
- **Solution**: Verify `docker-compose.yml` references correct `Dockerfile.webui`

---

## Build 3: Full Docker Compose Stack

**Purpose**: Build complete production stack (PostgreSQL + aw-server + aw-webui)

### Steps

#### 1. Clean Previous Builds (Optional)
```bash
docker compose down -v  # Remove containers and volumes
docker system prune -f  # Clean dangling images
```

**Warning**: This will delete all data in PostgreSQL volume. Only run for fresh start.

#### 2. Build All Services
```bash
docker compose build
```

**Expected Output**:
```
[+] Building XXXs (3/3) FINISHED
 => [postgresql] ...
 => [aw-server] ...
 => [aw-webui] ...
Successfully built postgresql, aw-server, aw-webui
```

**Build Time**: ~3-5 minutes (first build), ~1-2 minutes (cached)

#### 3. Verify All Images Created
```bash
docker images | grep -E "(postgres|aw-server|aw-webui)"
```

**Expected Output**:
```
aw-server-rust-aw-webui    latest   XXXXXXXXXXXX   X minutes ago   XX MB
aw-server-rust-aw-server   latest   XXXXXXXXXXXX   X minutes ago   XXX MB
postgres                   15-alpine XXXXXXXXXXXX   X weeks ago    XXX MB
```

### Success Criteria

✅ **Full Docker Compose build succeeds** when:
- [x] All 3 services build successfully
- [x] No build errors or warnings (critical)
- [x] All images tagged correctly
- [x] Build completes in reasonable time (< 10 minutes)

---

## Build Verification Checklist

### Pre-Build Checks
- [ ] `aw-webui/` directory contains all customized files
- [ ] `aw-webui/static/logo.png` exists (22.5 KB)
- [ ] `Dockerfile.webui` uses `COPY aw-webui` (not `git clone`)
- [ ] Node.js 20.x installed
- [ ] Docker and Docker Compose installed

### Local Build Checks
- [ ] `npm install` completes successfully
- [ ] `npm run build` completes without errors
- [ ] `dist/` directory created with all files
- [ ] `dist/logo.png` exists (22.5 KB PNG)
- [ ] Inter font import in `dist/index.html`

### Docker Build Checks
- [ ] `docker compose build aw-webui` succeeds
- [ ] WebUI image created (~50-80 MB)
- [ ] `docker compose build` (all services) succeeds
- [ ] All 3 images present (postgres, aw-server, aw-webui)

### Post-Build Validation
- [ ] No fatal errors in build logs
- [ ] Image sizes reasonable (no bloat)
- [ ] Build time acceptable (< 10 minutes total)

---

## Next Steps

After successful builds:
1. **Proceed to integration-test-instructions.md** - Start and test Docker stack
2. **Proceed to visual-verification-instructions.md** - Verify branding in browser
3. **Proceed to functional-test-instructions.md** - Test navigation and features

---

## Rollback Plan

If builds fail and cannot be fixed:

### 1. Restore Original Dockerfile
```bash
cp Dockerfile.webui.backup Dockerfile.webui
```

### 2. Remove Customized WebUI
```bash
rm -rf aw-webui/
```

### 3. Rebuild with Original
```bash
docker compose build aw-webui
```

This will clone the original ActivityWatch WebUI from GitHub.

---

**Build Instructions Complete**  
**Status**: Ready for Integration Testing
