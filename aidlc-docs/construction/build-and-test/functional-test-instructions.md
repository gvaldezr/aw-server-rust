# Functional Test Instructions - WebUI Customization

**Unit**: Unit 1 - WebUI Visual Customization  
**Date**: 2026-05-19  
**Purpose**: Verify all ActivityWatch functionality preserved after Anáhuac Mayab customization

---

## Functional Test Overview

These tests verify that visual customizations did NOT break any existing functionality.

**Critical Requirement**: NFR-2 (Functionality Preservation)
- ✅ All navigation must work
- ✅ All routes must be accessible
- ✅ All existing features must function
- ✅ No broken links or 404 errors

---

## Prerequisites

- ✅ Docker Compose stack running
- ✅ Visual verification passed
- ✅ WebUI accessible at http://localhost:5666

---

## Test 1: Navigation Links

### Steps

#### 1. Navigate to Home Page
```
http://localhost:5666
```

#### 2. Test Main Navigation Menu
**Location**: Top navigation bar

✅ **Test each link**:
- [ ] Click **Activity** → Navigates to `/activity/0/view` or similar
- [ ] Click **Timeline** → Navigates to `/timeline` or similar
- [ ] Click **Stopwatch** → Navigates to `/stopwatch`
- [ ] Click **Settings** → Navigates to `/settings`

**Expected**: Each link changes the page without errors

#### 3. Test Home Link/Logo
- [ ] From any page, click logo in header
- [ ] Returns to home page (`/`)

### Success Criteria

✅ **Navigation works** when:
- [x] All menu links navigate to correct pages
- [x] No 404 errors
- [x] No blank pages
- [x] Browser back button works
- [x] Logo returns to home

---

## Test 2: Home Page CTA Button

### Steps

#### 1. Click "Comenzar a Monitorear" Button
**Location**: Home page hero section

✅ **Check**:
- [ ] Button is clickable (not disabled)
- [ ] Clicking navigates to Activity view
- [ ] URL changes to `/activity/...`
- [ ] Activity page loads correctly
- [ ] No errors in browser console

### Success Criteria

✅ **CTA button works** when:
- [x] Button navigates to Activity view
- [x] Activity page renders correctly
- [x] No navigation errors

---

## Test 3: External Links in Resources Section

### Steps

#### 1. Locate Resources Section
**Location**: Bottom of home page

#### 2. Test Each External Link
✅ **Open in new tab (Ctrl+Click or Cmd+Click)**:

- [ ] **Sitio Web Oficial** → `https://activitywatch.net/`
  - Opens ActivityWatch official site
  
- [ ] **Documentación** → `https://activitywatch.readthedocs.org/`
  - Opens ActivityWatch docs
  
- [ ] **Foro de Soporte** → `https://forum.activitywatch.net/`
  - Opens ActivityWatch forum
  
- [ ] **GitHub** → `https://github.com/ActivityWatch/activitywatch`
  - Opens GitHub repository

- [ ] **API Browser** (if visible) → `/api/` or API explorer
  - Opens API documentation/browser

✅ **For each link**:
- [ ] Link opens in new tab
- [ ] Target site loads correctly
- [ ] No CORS errors (in browser console)

### Success Criteria

✅ **External links work** when:
- [x] All links are clickable
- [x] All links open correct destinations
- [x] Links open in new tabs (target="_blank")
- [x] No broken links (404 errors)

---

## Test 4: Activity View Functionality

### Steps

#### 1. Navigate to Activity View
Click "Activity" in menu or "Comenzar a Monitorear" button

#### 2. Verify Basic Elements Present
✅ **Check page elements**:
- [ ] Date selector visible
- [ ] Bucket selector visible (if data exists)
- [ ] Activity visualization area present
- [ ] No errors displayed

#### 3. Test Date Navigation (if functional)
- [ ] Click previous day
- [ ] Click next day
- [ ] Date picker works (if present)

**Note**: May show "No data" if no watchers running - this is acceptable

### Success Criteria

✅ **Activity view works** when:
- [x] Page loads without errors
- [x] UI elements render correctly
- [x] Date controls functional (if present)
- [x] No JavaScript errors in console

---

## Test 5: Timeline View Functionality

### Steps

#### 1. Navigate to Timeline View
Click "Timeline" in menu

#### 2. Verify Basic Elements Present
✅ **Check page elements**:
- [ ] Timeline visualization area present
- [ ] Date/time controls visible
- [ ] Bucket filters visible (if applicable)
- [ ] No errors displayed

#### 3. Test Timeline Interactions (if data present)
- [ ] Zoom in/out controls work (if present)
- [ ] Timeline scrolls horizontally/vertically
- [ ] Event details show on hover/click (if events exist)

**Note**: May show empty timeline if no data - acceptable

### Success Criteria

✅ **Timeline view works** when:
- [x] Page loads without errors
- [x] UI renders correctly
- [x] No JavaScript errors

---

## Test 6: Stopwatch Functionality

### Steps

#### 1. Navigate to Stopwatch View
Click "Stopwatch" in menu

#### 2. Verify Basic Elements Present
✅ **Check page elements**:
- [ ] Stopwatch timer display visible (00:00:00)
- [ ] Start/Stop button visible
- [ ] Activity/Category selectors visible
- [ ] No errors displayed

#### 3. Test Stopwatch Controls
- [ ] Click **Start** → Timer begins counting
- [ ] Click **Stop** → Timer stops
- [ ] Timer displays time correctly (HH:MM:SS)

### Success Criteria

✅ **Stopwatch works** when:
- [x] Page loads correctly
- [x] Start/Stop buttons functional
- [x] Timer counts accurately
- [x] No JavaScript errors

---

## Test 7: Settings Functionality

### Steps

#### 1. Navigate to Settings View
Click "Settings" in menu or gear icon

#### 2. Verify Settings Page Loads
✅ **Check page elements**:
- [ ] Settings form/options visible
- [ ] Various configuration options present
- [ ] Save button visible (if applicable)
- [ ] No errors displayed

#### 3. Test Setting Change (Non-Destructive)
- [ ] Change a non-critical setting (e.g., theme, language)
- [ ] Save changes (if applicable)
- [ ] Verify change persists on page refresh

**Note**: Be careful not to change production settings unless safe

### Success Criteria

✅ **Settings work** when:
- [x] Page loads correctly
- [x] Settings are editable
- [x] Changes save correctly (if applicable)
- [x] No JavaScript errors

---

## Test 8: Browser Back/Forward Navigation

### Steps

#### 1. Navigation Sequence
1. Start at Home (`/`)
2. Click "Activity" → Activity page
3. Click "Timeline" → Timeline page
4. Click "Stopwatch" → Stopwatch page

#### 2. Test Browser Back Button
- [ ] Click Back → Returns to Timeline
- [ ] Click Back → Returns to Activity
- [ ] Click Back → Returns to Home

#### 3. Test Browser Forward Button
- [ ] Click Forward → Returns to Activity
- [ ] Click Forward → Returns to Timeline
- [ ] Click Forward → Returns to Stopwatch

### Success Criteria

✅ **Browser navigation works** when:
- [x] Back button navigates to previous pages
- [x] Forward button navigates to next pages
- [x] URL updates correctly
- [x] Page state restored correctly

---

## Test 9: Page Refresh Persistence

### Steps

#### 1. Navigate to Each View
For each view (Activity, Timeline, Stopwatch, Settings):

- [ ] Navigate to the view
- [ ] Press `F5` or `Ctrl+R` (Cmd+R on Mac) to refresh
- [ ] Verify page reloads successfully
- [ ] Verify URL remains the same
- [ ] Verify content renders correctly

### Success Criteria

✅ **Page refresh works** when:
- [x] All pages reload successfully
- [x] URLs persist after refresh
- [x] Content reappears correctly
- [x] No 404 or blank pages

---

## Test 10: Error Handling (No Data Scenarios)

### Steps

#### 1. Test Views with No Data
If no watchers are running and no data exists:

✅ **Check each view shows appropriate messages**:
- [ ] Activity view shows "No data" or helpful message
- [ ] Timeline view shows empty timeline or guide
- [ ] No crashes or blank pages
- [ ] User-friendly error messages (not raw errors)

### Success Criteria

✅ **Error handling works** when:
- [x] Views handle empty data gracefully
- [x] Helpful messages shown (not crashes)
- [x] UI remains functional with no data

---

## Test 11: Console Error Check

### Steps

#### 1. Open Browser Developer Console
**Shortcut**: `F12` or `Cmd+Option+I` (Mac)

#### 2. Navigate Through All Views
Visit each page: Home → Activity → Timeline → Stopwatch → Settings

#### 3. Monitor Console for Errors
✅ **Check for errors**:
- [ ] No `Uncaught TypeError` errors
- [ ] No `404 Not Found` errors for resources
- [ ] No `Failed to load resource` errors
- [ ] No CORS errors
- [ ] Info/warnings acceptable (not errors)

### Success Criteria

✅ **Console is clean** when:
- [x] No critical JavaScript errors
- [x] No missing resource errors (404s)
- [x] No CORS policy violations
- [x] Application runs without crashes

---

## Functional Test Checklist

### Navigation Tests
- [ ] All main menu links work (Activity, Timeline, Stopwatch, Settings)
- [ ] Logo/home link returns to home page
- [ ] CTA button navigates to Activity view
- [ ] External links open correctly (in new tabs)
- [ ] Browser back/forward buttons work
- [ ] Page URLs update correctly

### Feature Tests
- [ ] Activity view loads and renders
- [ ] Timeline view loads and renders
- [ ] Stopwatch starts/stops correctly
- [ ] Settings page loads and is editable
- [ ] All views handle no-data scenarios gracefully

### Interaction Tests
- [ ] Button hover effects work (visual + functional)
- [ ] Clickable elements respond to clicks
- [ ] Form inputs accept input (if present)
- [ ] Dropdowns/selectors work (if present)

### Error Handling Tests
- [ ] No JavaScript errors in console
- [ ] No 404 errors for resources
- [ ] No CORS errors
- [ ] Graceful handling of empty data
- [ ] User-friendly error messages

### Persistence Tests
- [ ] Page refresh doesn't break functionality
- [ ] URL state persists
- [ ] Settings changes persist (if applicable)

---

## Regression Test: Verify No Changes to Backend

### Critical Verification

#### 1. Verify Queries Still Work
```bash
curl -s http://localhost:5600/api/0/buckets | jq 'keys'
```

✅ **Expected**: Returns bucket list (may be empty array `[]`)

#### 2. Verify No Backend Modifications
✅ **Confirm**:
- [ ] aw-server code NOT modified (only Dockerfile and config)
- [ ] aw-datastore code NOT modified
- [ ] PostgreSQL queries NOT modified
- [ ] API endpoints NOT modified
- [ ] Backend logs show no errors

**Critical**: This customization is **frontend-only**. Backend must be untouched.

---

## Test Summary Template

After completing all tests, document results:

```markdown
## Functional Test Results

**Date**: 2026-05-19  
**Tester**: [Name]  
**Browser**: [Chrome/Firefox/Safari/Edge] [Version]

### Test Results
- Navigation Tests: ✅ PASS / ❌ FAIL
- Feature Tests: ✅ PASS / ❌ FAIL
- Interaction Tests: ✅ PASS / ❌ FAIL
- Error Handling: ✅ PASS / ❌ FAIL
- Console Errors: ✅ PASS / ❌ FAIL

### Issues Found
1. [Description of any issues]
2. [Another issue if any]

### Overall Status
✅ All tests passed - Ready for production
❌ Issues found - Requires fixes
```

---

## Next Steps

After successful functional tests:
1. **Proceed to performance-test-instructions.md** - Measure page load performance
2. **Document results** in build-and-test-summary.md
3. **Report any issues** for resolution

---

**Functional Test Instructions Complete**  
**Status**: Ready for Performance Testing
