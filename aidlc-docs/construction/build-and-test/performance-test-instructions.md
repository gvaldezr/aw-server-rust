# Performance Test Instructions - WebUI Customization

**Unit**: Unit 1 - WebUI Visual Customization  
**Date**: 2026-05-19  
**Purpose**: Verify performance requirements met (page load < 2 seconds)

---

## Performance Test Overview

These tests verify that visual customizations did NOT degrade performance.

**Critical Requirement**: NFR-3 (Performance < 2 seconds page load)
- ✅ Home page loads in < 2 seconds
- ✅ Inter font loads efficiently
- ✅ Logo loads quickly (~22.5 KB)
- ✅ No performance regressions

---

## Prerequisites

- ✅ Docker Compose stack running
- ✅ Browser installed (Chrome recommended for best dev tools)
- ✅ Network connection stable

---

## Test 1: Home Page Load Time (Browser DevTools)

### Steps

#### 1. Open Chrome DevTools
- Open http://localhost:5666
- Press `F12` or `Cmd+Option+I` (Mac)
- Go to **Network** tab

#### 2. Enable Performance Metrics
- Check "Disable cache" checkbox (for consistent results)
- Throttling: "No throttling" (local test) or "Fast 3G" (realistic)

#### 3. Reload Page and Measure
- Press `Ctrl+R` or `Cmd+R` to refresh
- Wait for page to fully load

#### 4. Check Load Metrics
**Look for these metrics at bottom of Network tab**:

✅ **Check values**:
- [ ] **DOMContentLoaded**: < 1.5 seconds (critical)
- [ ] **Load**: < 2 seconds (critical - NFR requirement)
- [ ] **Finish**: < 2.5 seconds (acceptable)
- [ ] **Total requests**: ~10-30 requests
- [ ] **Total transfer**: < 500 KB

### Success Criteria

✅ **Performance meets requirements** when:
- [x] **Load event < 2 seconds** (CRITICAL - NFR-3)
- [x] DOMContentLoaded < 1.5 seconds
- [x] First Contentful Paint < 1 second
- [x] Largest Contentful Paint < 2 seconds

### Expected Metrics (Baseline)
- **DOMContentLoaded**: 0.5 - 1.0 seconds
- **Load**: 1.0 - 1.8 seconds
- **Total Size**: 200-400 KB
- **Requests**: 15-25

---

## Test 2: Resource Loading Analysis

### Steps

#### 1. Inspect Network Tab (After Page Load)
Sort by **Size** (descending) to find largest resources

#### 2. Verify Key Resources
✅ **Check resource sizes**:

- [ ] **index.html**: < 10 KB
- [ ] **logo.png**: ~22-23 KB (optimized)
- [ ] **JavaScript bundles**: 50-200 KB (compressed)
- [ ] **CSS bundles**: 20-100 KB (compressed)
- [ ] **Inter font files (.woff2)**: ~10-30 KB each (3 weights)
- [ ] **Total fonts**: < 100 KB

#### 3. Check Resource Load Times
✅ **Individual resource times** (in Network tab):
- [ ] logo.png: < 100 ms
- [ ] Inter fonts: < 200 ms each
- [ ] JavaScript: < 500 ms
- [ ] CSS: < 200 ms

### Success Criteria

✅ **Resource loading is efficient** when:
- [x] Logo optimized (~22.5 KB, not > 50 KB)
- [x] Fonts load quickly (< 200 ms each)
- [x] No resources over 500 KB
- [x] Total page weight < 500 KB

---

## Test 3: Lighthouse Performance Audit

### Steps

#### 1. Open Chrome DevTools → Lighthouse Tab
- DevTools → **Lighthouse** tab (top menu)
- If not visible, click `>>` and select Lighthouse

#### 2. Configure Audit
✅ **Settings**:
- [x] Mode: **Navigation** (default)
- [x] Device: **Desktop**
- [x] Categories: Check **Performance** only (faster)
- [x] Throttling: **Applied** (Simulated throttling)

#### 3. Run Audit
- Click **Analyze page load**
- Wait 30-60 seconds for audit to complete

#### 4. Review Performance Score
✅ **Check Lighthouse scores**:
- [ ] **Performance Score**: > 90 (excellent) / > 80 (good) / > 70 (acceptable)
- [ ] **First Contentful Paint (FCP)**: < 1.5s
- [ ] **Largest Contentful Paint (LCP)**: < 2.5s
- [ ] **Total Blocking Time (TBT)**: < 300ms
- [ ] **Cumulative Layout Shift (CLS)**: < 0.1

#### 5. Check Opportunities Section
✅ **Review recommendations**:
- [ ] No critical issues (red)
- [ ] Minor optimizations acceptable (yellow/green)

### Success Criteria

✅ **Lighthouse audit passes** when:
- [x] Performance score > 70 (acceptable) or > 90 (excellent)
- [x] No critical performance issues flagged
- [x] Core Web Vitals in green/yellow ranges

### Expected Results
- **Performance Score**: 85-100 (excellent)
- **FCP**: 0.5-1.2 seconds
- **LCP**: 1.0-2.0 seconds
- **CLS**: < 0.05 (minimal layout shift)

---

## Test 4: Font Loading Performance

### Steps

#### 1. Check Font Loading Strategy
- Open DevTools → **Network** tab
- Filter by "Font"
- Reload page

#### 2. Verify Font Load Waterfall
✅ **Check fonts**:
- [ ] Inter-Regular (400) loaded
- [ ] Inter-Medium (500) loaded
- [ ] Inter-Bold (700) loaded
- [ ] Fonts load after CSS (acceptable)
- [ ] No FOIT (Flash of Invisible Text) - text visible while fonts load

#### 3. Verify Preconnect Optimization
- Network tab → Filter by "Other" or "Doc"
- Look for early connections to `fonts.googleapis.com`

✅ **Expected**:
- [ ] Preconnect requests sent early
- [ ] DNS resolution happens before font CSS request

### Success Criteria

✅ **Font loading is optimized** when:
- [x] Preconnect to fonts.googleapis.com working
- [x] `display=swap` prevents invisible text
- [x] All 3 font weights load in < 500 ms total
- [x] No double-loading of fonts

---

## Test 5: Image Optimization (Logo)

### Steps

#### 1. Verify Logo File Size
```bash
ls -lh /Users/guillermo.valdez/Documents/dti-timetracker-apps/aw-rust/aw-server-rust/aw-webui/static/logo.png
```

✅ **Expected**:
```
-rw-r--r--  1 user  staff   22K May 19 01:00 logo.png
```

**Critical**: File size ~22-23 KB (not > 50 KB)

#### 2. Verify Logo Network Performance
- DevTools → **Network** tab
- Filter by "Img"
- Find `logo.png` request

✅ **Check metrics**:
- [ ] Size: ~22 KB
- [ ] Load time: < 100 ms (local) / < 500 ms (remote)
- [ ] Status: 200 OK
- [ ] No 404 errors

#### 3. Verify Logo Dimensions
- Right-click logo on page
- Inspect element
- Check `naturalWidth` and `naturalHeight` in console:
  ```javascript
  $0.naturalWidth  // Should be 512
  $0.naturalHeight // Should be 512
  ```

✅ **Expected**: 512x512 px (high quality for retina displays)

### Success Criteria

✅ **Logo is optimized** when:
- [x] File size ~22.5 KB (< 50 KB requirement met)
- [x] Dimensions 512x512 (good for 2x retina @ 256px display)
- [x] PNG with transparency (alpha channel)
- [x] Loads in < 500 ms

---

## Test 6: Caching and Repeat Load Performance

### Steps

#### 1. First Load (Cache Empty)
- DevTools → Network tab
- "Disable cache" **unchecked**
- **Hard refresh**: `Ctrl+Shift+R` or `Cmd+Shift+R`
- Note **Load** time: _______ seconds

#### 2. Second Load (Cache Primed)
- Keep DevTools open
- Normal refresh: `Ctrl+R` or `Cmd+R`
- Note **Load** time: _______ seconds

#### 3. Compare Load Times
✅ **Expected**:
- [ ] Second load significantly faster (50-80% reduction)
- [ ] Second load: < 1 second
- [ ] Resources served from cache (200 OK, from cache)

### Success Criteria

✅ **Caching works** when:
- [x] Repeat load much faster than first load
- [x] Static assets cached (logo, CSS, JS)
- [x] Cache headers set correctly

---

## Test 7: Slow Network Simulation

### Steps

#### 1. Simulate Slow 3G Network
- DevTools → **Network** tab
- Throttling dropdown → **Slow 3G**

#### 2. Reload Page
- Press `Ctrl+R` or `Cmd+R`
- Wait for page to load

#### 3. Measure Load Time
✅ **Check**:
- [ ] Page loads eventually (within 10 seconds)
- [ ] Content visible progressively (not blank until done)
- [ ] Text visible before images (acceptable)
- [ ] Inter font swap prevents invisible text

#### 4. Reset Throttling
- Throttling → **No throttling**

### Success Criteria

✅ **Slow network handled** when:
- [x] Page loads within 10 seconds on Slow 3G
- [x] Progressive rendering works
- [x] No blank page while loading
- [x] Text visible immediately (system fonts as fallback)

---

## Test 8: Memory and CPU Usage

### Steps

#### 1. Open DevTools → Performance Tab

#### 2. Record Page Load
- Click **Record** button (circle)
- Reload page (`Ctrl+R`)
- Wait for load to complete
- Click **Stop** button

#### 3. Analyze Recording
✅ **Check metrics**:
- [ ] CPU usage spikes briefly then drops (normal)
- [ ] No long tasks (> 50 ms) blocking main thread
- [ ] Memory usage stable (no leaks)
- [ ] Scripting time < 500 ms
- [ ] Rendering time < 200 ms

### Success Criteria

✅ **Performance profile is healthy** when:
- [x] No long blocking tasks (> 50 ms)
- [x] CPU usage returns to idle after load
- [x] Memory usage stable (no continuous growth)
- [x] Main thread not blocked during load

---

## Performance Test Checklist

### Load Time Tests
- [ ] Home page loads in < 2 seconds (NFR-3 CRITICAL)
- [ ] DOMContentLoaded < 1.5 seconds
- [ ] First Contentful Paint < 1 second
- [ ] Largest Contentful Paint < 2.5 seconds

### Resource Efficiency Tests
- [ ] Logo file size ~22.5 KB (< 50 KB)
- [ ] Total page weight < 500 KB
- [ ] JavaScript bundle < 200 KB (compressed)
- [ ] CSS bundle < 100 KB (compressed)
- [ ] Inter fonts < 100 KB total

### Font Performance Tests
- [ ] Preconnect to fonts.googleapis.com works
- [ ] display=swap prevents invisible text (FOIT)
- [ ] All font weights load in < 500 ms
- [ ] No font loading errors

### Optimization Tests
- [ ] Caching works (repeat load < 1 second)
- [ ] Static assets cached correctly
- [ ] Slow 3G loads within 10 seconds
- [ ] Progressive rendering works

### Lighthouse Tests
- [ ] Performance score > 70 (acceptable) or > 90 (excellent)
- [ ] No critical performance issues
- [ ] Core Web Vitals in acceptable ranges

### Profiling Tests
- [ ] No long blocking tasks (> 50 ms)
- [ ] CPU usage returns to idle
- [ ] Memory usage stable
- [ ] Main thread responsive

---

## Performance Benchmarks

### Target Performance (NFR-3)
- **Page Load**: < 2 seconds ✅ CRITICAL
- **FCP**: < 1.5 seconds
- **LCP**: < 2.5 seconds
- **CLS**: < 0.1
- **TBT**: < 300 ms

### Expected Results (Local Docker)
- **Page Load**: 1.0-1.8 seconds ✅
- **FCP**: 0.5-1.0 seconds ✅
- **LCP**: 1.0-2.0 seconds ✅
- **Lighthouse**: 85-100 score ✅

### Acceptable Results
- **Page Load**: < 2 seconds (meets NFR-3)
- **Lighthouse**: > 70 score
- **No critical issues**

---

## Performance Test Results Template

```markdown
## Performance Test Results

**Date**: 2026-05-19  
**Tester**: [Name]  
**Browser**: Chrome [Version]  
**Network**: Local (No throttling)

### Load Time Metrics
- DOMContentLoaded: [X.XX] seconds
- Load Event: [X.XX] seconds ← CRITICAL (must be < 2s)
- First Contentful Paint: [X.XX] seconds
- Largest Contentful Paint: [X.XX] seconds

### Resource Metrics
- Total Size: [XXX] KB
- Logo Size: [XX.X] KB
- Fonts Size: [XX] KB
- JavaScript: [XXX] KB
- CSS: [XX] KB

### Lighthouse Score
- Performance: [XX] / 100
- FCP: [X.XX] seconds
- LCP: [X.XX] seconds
- CLS: [X.XXX]
- TBT: [XXX] ms

### NFR-3 Compliance
✅ PASS - Page load < 2 seconds
❌ FAIL - Page load > 2 seconds (requires optimization)

### Issues Found
1. [Any performance issues]
2. [Optimization recommendations]

### Overall Status
✅ Performance requirements met (NFR-3 compliant)
❌ Performance issues found (requires fixes)
```

---

## Next Steps

After successful performance tests:
1. **Document results** in build-and-test-summary.md
2. **Compare with original ActivityWatch** (optional benchmark)
3. **Report any performance regressions** for optimization

---

## Troubleshooting

### Page Load > 2 Seconds

**Possible causes**:
- Logo file too large → Re-optimize image
- Too many font weights → Reduce to essential weights only
- Large JavaScript bundles → Check Vite build optimization
- Network latency → Test on local network vs remote

**Solutions**:
1. Verify logo is optimized 22.5 KB PNG
2. Check only 3 font weights loaded (400, 500, 700)
3. Enable Vite production optimizations
4. Use browser caching headers

### Fonts Load Slowly

**Solutions**:
- Verify preconnect links in index.html
- Check `display=swap` parameter in font URL
- Consider hosting fonts locally (optional)

### Low Lighthouse Score

**Common issues**:
- Unused CSS/JS → Review Vite tree-shaking
- Large images → Verify logo optimized
- No caching → Check nginx cache headers

---

**Performance Test Instructions Complete**  
**Status**: Ready for Final Summary
