# Fix #6: Timeline queriedInterval Null Safety

**Date**: 2026-05-20  
**Component**: aw-webui (Vue.js Frontend)  
**File**: `aw-webui/src/visualizations/VisTimeline.vue`  
**Issue Type**: TypeError - Null Reference  
**Severity**: High (Page crash)

---

## 🐛 Problem Description

### User Report
```
TypeError: undefined is not an object (evaluating 'this.queriedInterval[0]')
See dev console (F12) and/or server logs for more info.
```

### Root Cause Analysis

The `VisTimeline.vue` component was accessing `queriedInterval[0]` and `queriedInterval[1]` without verifying that:
1. The `queriedInterval` property is not null/undefined
2. Both array indices exist and contain valid values

**Critical Code Locations**:

**Location 1** - Line 296 (Incomplete null check):
```javascript
if (this.queriedInterval && this.showQueriedInterval) {
  const duration = this.queriedInterval[1].diff(this.queriedInterval[0], 'seconds');
  // ... more code accessing queriedInterval[0] and queriedInterval[1]
}
```
**Problem**: Checks if `queriedInterval` exists but doesn't verify array indices are populated.

**Location 2** - Lines 342-345 (No null check):
```javascript
} else {
  // update the timeline range
  this.options.min = this.queriedInterval[0];
  this.options.max = this.queriedInterval[1];
  this.timeline.setOptions(this.options);
  this.timeline.setWindow(this.queriedInterval[0], this.queriedInterval[1]);
```
**Problem**: Directly accesses `queriedInterval[0]` and `[1]` without ANY null checks.

### Error Chain

1. Component initializes with `queriedInterval` as null or undefined
2. Component enters else block (no groups/items to display)
3. Code attempts: `this.queriedInterval[0]` when queriedInterval is null
4. `null[0]` returns `undefined`
5. Attempting to use `undefined` as a timeline parameter throws `TypeError`

### Impact

- **User Experience**: Timeline view crashes when no events exist
- **Frequency**: Occurs on empty buckets or initial page loads
- **Visibility**: Immediate - user sees error message instead of timeline

---

## ✅ Solution

### Changes Made

**File**: `aw-webui/src/visualizations/VisTimeline.vue`

**CHANGE 1** - Enhanced null check (line 296):
```javascript
// BEFORE
if (this.queriedInterval && this.showQueriedInterval) {

// AFTER
if (this.queriedInterval && this.queriedInterval[0] && this.queriedInterval[1] && this.showQueriedInterval) {
```

**CHANGE 2** - Added null check in else block (lines 342-350):
```javascript
// BEFORE
} else {
  // update the timeline range
  this.options.min = this.queriedInterval[0];
  this.options.max = this.queriedInterval[1];
  this.timeline.setOptions(this.options);
  this.timeline.setWindow(this.queriedInterval[0], this.queriedInterval[1]);

  // clear the data
  this.timeline.setData({ groups: [], items: [] });

// AFTER
} else {
  // update the timeline range (only if queriedInterval is valid)
  if (this.queriedInterval && this.queriedInterval[0] && this.queriedInterval[1]) {
    this.options.min = this.queriedInterval[0];
    this.options.max = this.queriedInterval[1];
    this.timeline.setOptions(this.options);
    this.timeline.setWindow(this.queriedInterval[0], this.queriedInterval[1]);
  }

  // clear the data
  this.timeline.setData({ groups: [], items: [] });
```

### Why This Works

1. **Complete Validation**: Checks both `queriedInterval` existence AND array indices
2. **Safe Access**: Only accesses array elements after confirming they exist
3. **Graceful Degradation**: Timeline still clears data even if queriedInterval is invalid
4. **Defense in Depth**: Multiple layers of protection for robustness
5. **Consistent Pattern**: Matches null safety pattern used elsewhere in the component

### Testing Strategy

1. **Empty Bucket Test**: Navigate to bucket with no events
2. **Initial Load Test**: Load timeline before queriedInterval is set
3. **Date Range Test**: Verify timeline updates correctly when valid daterange is provided
4. **Console Verification**: Confirm no TypeError appears in browser console

---

## 📊 Verification Steps

### Pre-Fix Behavior
- Navigate to bucket detail page
- Timeline crashes with TypeError
- Console shows: `TypeError: undefined is not an object (evaluating 'this.queriedInterval[0]')`

### Post-Fix Behavior
- Navigate to bucket detail page
- Timeline displays empty (no crash)
- No TypeError in console
- Timeline populates correctly once queriedInterval becomes valid

### Verification Commands
```bash
# 1. Rebuild WebUI
cd aw-webui && npm run build

# 2. Rebuild Docker image
cd .. && docker compose build aw-webui

# 3. Restart container
docker compose restart aw-webui

# 4. Verify service health
docker compose ps | grep webui
curl -s -o /dev/null -w "Status: %{http_code}\n" http://localhost:8080
```

### Manual Testing
1. Open http://localhost:8080
2. **CRITICAL**: Perform hard refresh (CMD+SHIFT+R or CTRL+SHIFT+R)
3. Navigate to any bucket detail page
4. Open browser console (F12)
5. Verify: No TypeError appears
6. Verify: Timeline displays correctly (empty or with events)

---

## 🔧 Technical Details

### Related Code Patterns

This fix follows the same null safety pattern applied to other date/time range properties:

**Pattern**:
```javascript
if (this.property && this.property[0] && this.property[1]) {
  // Safe to access property[0] and property[1]
}
```

**Applied to**:
- `daterange` (in Bucket.vue - Fix #5)
- `queriedInterval` (in VisTimeline.vue - Fix #6)

### Component Lifecycle

1. **Mount**: Component initializes with null queriedInterval
2. **Props Update**: Parent component sets queriedInterval
3. **Watch Trigger**: Component reacts to queriedInterval changes
4. **Safe Access**: Null checks prevent crashes during initialization

### Dependencies

- **Property Type**: `Array<Moment>` (two Moment.js objects)
- **Parent Components**: Activity, Bucket, Stopwatch views
- **Used By**: Timeline visualization rendering
- **Related**: daterange property (similar but different use case)

---

## 📝 Related Fixes

This fix is part of a series addressing null safety issues:

- **Fix #5**: `Bucket.vue` - daterange null safety (watcher + getEvents method)
- **Fix #6**: `VisTimeline.vue` - queriedInterval null safety (this fix)

Both fixes address the same pattern: date/time range arrays that can be null during component initialization.

---

## 🚀 Deployment

### Build Results
- **Build Time**: ~13.7 seconds (npm) + ~20 seconds (Docker)
- **Bundle Changes**: VisTimeline component recompiled
- **Warnings**: 9 prettier warnings (spacing, non-critical)
- **Status**: ✅ Successful deployment

### Deployment Steps Taken
1. Modified VisTimeline.vue with null checks
2. Built WebUI with `npm run build`
3. Built Docker image with `docker compose build aw-webui`
4. Restarted container with `docker compose restart aw-webui`
5. Verified HTTP 200 response on port 8080

### Rollback Plan (if needed)
```bash
# Restore from backup
cp aw-webui/src/visualizations/VisTimeline.vue.backup \
   aw-webui/src/visualizations/VisTimeline.vue

# Rebuild and redeploy
cd aw-webui && npm run build
cd .. && docker compose build aw-webui
docker compose restart aw-webui
```

---

## 📋 Checklist

- [x] Root cause identified
- [x] Fix implemented (2 locations)
- [x] Code backup created
- [x] Unit of work built successfully
- [x] Docker image built successfully
- [x] Container restarted
- [x] HTTP endpoint verified (200 OK)
- [x] Documentation created
- [x] Audit log updated
- [ ] User verification pending (requires hard refresh)

---

## 🎯 Success Criteria

- [ ] No TypeError in browser console
- [ ] Timeline displays correctly on empty buckets
- [ ] Timeline displays correctly on populated buckets
- [ ] Timeline responds to queriedInterval updates
- [ ] No regression in existing timeline functionality

---

## 📌 Notes

- **Hard Refresh Required**: Users MUST perform hard refresh (CMD+SHIFT+R or CTRL+SHIFT+R) to load new JavaScript bundle
- **Non-Breaking**: Fix is backwards compatible - existing functionality unchanged
- **Performance**: No performance impact - adds minimal null checks
- **Maintainability**: Improves code robustness and reduces crash potential
