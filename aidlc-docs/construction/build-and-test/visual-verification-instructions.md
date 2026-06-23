# Visual Verification Instructions - Anáhuac Mayab Branding

**Unit**: Unit 1 - WebUI Visual Customization  
**Date**: 2026-05-19  
**Purpose**: Verify Anáhuac Mayab branding displays correctly in browser

---

## Visual Verification Overview

These tests verify that all visual customizations (logo, colors, typography, home page) render correctly in the browser.

**Test Scope**:
1. Logo display (header + home page)
2. Color palette (Naranja Anáhuac #FF5900)
3. Typography (Inter font)
4. Home page content (Anáhuac Mayab messaging)
5. Button styles (hover effects)
6. Responsive design (desktop, tablet, mobile)

---

## Prerequisites

- ✅ Docker Compose stack running (see integration-test-instructions.md)
- ✅ Browser installed (Chrome, Firefox, Safari, or Edge)
- ✅ WebUI accessible at http://localhost:5666

---

## Test 1: Home Page Visual Elements

### Steps

#### 1. Open WebUI in Browser
```
http://localhost:5666
```

**Browser**: Chrome, Firefox, Safari, or Edge (latest version)

#### 2. Verify Logo (Hero Section)
**Location**: Top center of home page

✅ **Check**:
- [ ] Anáhuac Mayab isotipo (black circular logo) visible
- [ ] Logo renders sharp and clear (no pixelation)
- [ ] Logo has white/light background container
- [ ] Logo height ~120px on desktop
- [ ] Logo has subtle shadow effect

**Expected**: Black Anáhuac isotipo on light background, centered above title

#### 3. Verify Title and Subtitle
**Location**: Below logo in hero section

✅ **Check**:
- [ ] Title: "ActivityWatch - Anáhuac Mayab" (large, bold)
- [ ] Subtitle: "Sistema de Monitoreo de Uso de Software" (medium)
- [ ] Title color: Naranja Anáhuac (#FF5900) or similar
- [ ] Text uses Inter font (clean, modern sans-serif)
- [ ] Text center-aligned

**Expected**: Two lines of Spanish text, Anáhuac colors, Inter font

#### 4. Verify Description Text
**Location**: Below subtitle

✅ **Check**:
- [ ] Text: "Bienvenido al sistema de seguimiento de software..."
- [ ] Color: Medium gray (not pure black)
- [ ] Font: Inter
- [ ] Text center-aligned and readable

#### 5. Verify CTA Button
**Location**: Below description text

✅ **Check**:
- [ ] Button text: "Comenzar a Monitorear"
- [ ] Button background: Naranja Anáhuac (#FF5900)
- [ ] Button text: White
- [ ] Button has rounded corners (~8px)
- [ ] Button medium-large size (padding visible)

**Hover Test**:
- [ ] Hover over button
- [ ] Background darkens slightly (#E04F00)
- [ ] Button lifts slightly (translateY effect)
- [ ] Shadow appears below button
- [ ] Smooth transition (~0.3s)

**Click Test**:
- [ ] Click button
- [ ] Navigates to Activity view (`/activity/0/view` or similar)

---

## Test 2: Header Logo

### Steps

#### 1. Verify Logo in Header/Navbar
**Location**: Top-left corner of page

✅ **Check**:
- [ ] Anáhuac Mayab isotipo visible in navbar
- [ ] Logo height ~1.5em (smaller than home page logo)
- [ ] Logo renders sharp (not blurry)
- [ ] Logo aligned with "ActivityWatch" text

**Expected**: Small Anáhuac logo next to ActivityWatch branding

#### 2. Navigation Test
- [ ] Click logo in header
- [ ] Returns to home page

---

## Test 3: Feature Cards

### Steps

#### 1. Locate Feature Cards Section
**Location**: Below hero section, after horizontal rule

✅ **Check**: 3 cards in a row (desktop):
- [ ] "Monitoreo en Tiempo Real" (left)
- [ ] "Privacidad Local" (center)
- [ ] "Reportes Detallados" (right)

#### 2. Verify Card Styling
✅ **Check each card**:
- [ ] White background
- [ ] Border visible (light gray)
- [ ] Rounded corners (~12px)
- [ ] Icon at top (chart/shield/pie icons)
- [ ] Icon color: Naranja Anáhuac (#FF5900)
- [ ] Title below icon (bold, Inter font)
- [ ] Description text (gray, Inter font)
- [ ] Consistent spacing and padding

#### 3. Hover Test (Each Card)
- [ ] Hover over card
- [ ] Card lifts up slightly (translateY effect)
- [ ] Shadow appears/intensifies below card
- [ ] Border color may lighten
- [ ] Smooth transition (~0.3s)

---

## Test 4: Color Palette Verification

### Steps

#### 1. Open Browser Developer Tools
**Shortcut**: `F12` or `Cmd+Option+I` (Mac) or `Ctrl+Shift+I` (Windows/Linux)

#### 2. Inspect CTA Button Element
- Right-click "Comenzar a Monitorear" button
- Select "Inspect Element"
- Check `Computed` tab for `background-color`

✅ **Expected**:
```
background-color: rgb(255, 89, 0)
```
**Hex equivalent**: #FF5900 (Naranja Anáhuac)

#### 3. Inspect Title Color
- Right-click title "ActivityWatch - Anáhuac Mayab"
- Inspect element
- Check `color` property

✅ **Expected**: 
```
color: rgb(255, 89, 0) or similar orange shade
```

#### 4. Verify Secondary Colors (Optional)
- Border colors should be light gray (#E0E0E0)
- Text colors: #040404 (primary), #262626 (secondary), #9C9C9C (muted)
- Background: #FFFFFF (white)

---

## Test 5: Typography Verification

### Steps

#### 1. Open Developer Tools → Elements Tab

#### 2. Inspect Body Element
- Right-click anywhere on page
- Inspect element
- Navigate up to `<body>` tag
- Check `Computed` tab for `font-family`

✅ **Expected**:
```
font-family: Inter, "Segoe UI", -apple-system, BlinkMacSystemFont, sans-serif
```

**Critical**: `Inter` should be first in the stack

#### 3. Verify Font Loads
- Open `Network` tab in Developer Tools
- Filter by `Font` or `css`
- Look for `fonts.googleapis.com/css2?family=Inter`

✅ **Expected**:
- [ ] Google Fonts request present
- [ ] Status: 200 (success)
- [ ] Font files (.woff2) loaded

#### 4. Visual Font Check
✅ **Compare text rendering**:
- [ ] Text looks modern and clean (not Times New Roman or Arial)
- [ ] Consistent font across all text elements
- [ ] Bold text (title) distinctly bolder
- [ ] Text anti-aliased and smooth

---

## Test 6: Border Radius Consistency

### Steps

#### 1. Inspect Element Border Radius
Using Developer Tools, check border-radius on:

✅ **CTA Button**:
```
border-radius: 8px
```

✅ **Feature Cards**:
```
border-radius: 12px
```

✅ **Logo Container**:
```
border-radius: 12px
```

✅ **Hero Section**:
```
border-radius: 12px
```

**Expected**: Consistent use of 4px (small), 8px (medium), 12px (large)

---

## Test 7: Responsive Design (Mobile)

### Steps

#### 1. Open Responsive Design Mode
**Chrome/Edge**: `Ctrl+Shift+M` or `Cmd+Option+M` (Mac)  
**Firefox**: `Ctrl+Shift+M` or `Cmd+Option+M` (Mac)

#### 2. Test Mobile View (375x667 - iPhone SE)
✅ **Check**:
- [ ] Logo scales down to ~80px height
- [ ] Title font size reduces (remains readable)
- [ ] Hero section padding adjusts (narrower)
- [ ] Feature cards stack vertically (one per row)
- [ ] Button remains full-width or well-sized
- [ ] Text wraps properly (no overflow)
- [ ] No horizontal scrolling

#### 3. Test Tablet View (768x1024 - iPad)
✅ **Check**:
- [ ] Layout between mobile and desktop
- [ ] Feature cards may be 2 columns or stacked
- [ ] Logo and text scale appropriately
- [ ] Spacing adjusts gracefully

#### 4. Test Desktop View (1920x1080 - Full HD)
✅ **Check**:
- [ ] Logo 120px height
- [ ] Feature cards in 3-column row
- [ ] Hero section has generous padding (4rem)
- [ ] Content centered and not stretched too wide
- [ ] All elements visible without scrolling (hero section)

---

## Visual Verification Checklist

### Logo Tests
- [ ] Home page logo visible (120px, Anáhuac isotipo, sharp rendering)
- [ ] Header logo visible (1.5em, top-left corner)
- [ ] Logo has proper background/container
- [ ] Logo renders on light and dark backgrounds correctly

### Color Tests
- [ ] Primary color: #FF5900 (Naranja Anáhuac) on button
- [ ] Hover color: #E04F00 (darker orange)
- [ ] Text colors: Black/gray variants as designed
- [ ] Border colors: Light gray (#E0E0E0)
- [ ] Background: White (#FFFFFF)

### Typography Tests
- [ ] Inter font loads from Google Fonts
- [ ] Inter font applied to all text (body, headings, buttons)
- [ ] Font weights: 400 (regular), 500 (medium), 700 (bold)
- [ ] Text anti-aliased and smooth
- [ ] Consistent typography across all elements

### Home Page Content Tests
- [ ] Title: "ActivityWatch - Anáhuac Mayab"
- [ ] Subtitle: "Sistema de Monitoreo de Uso de Software"
- [ ] Description text present and correct
- [ ] CTA button: "Comenzar a Monitorear"
- [ ] 3 feature cards visible
- [ ] Resources section visible

### Interaction Tests
- [ ] Button hover effect works (darken + lift + shadow)
- [ ] Button click navigates correctly
- [ ] Card hover effects work (lift + shadow)
- [ ] Logo click returns to home
- [ ] Smooth transitions (~0.3s)

### Responsive Tests
- [ ] Mobile (375x667): Logo 80px, cards stack, no overflow
- [ ] Tablet (768x1024): Layout adapts gracefully
- [ ] Desktop (1920x1080): Logo 120px, 3-column cards, proper spacing
- [ ] No horizontal scrolling on any device

### Browser Console Tests
- [ ] No JavaScript errors in console
- [ ] No missing resource errors (404s)
- [ ] No CSS warnings (critical)
- [ ] Google Fonts load successfully

---

## Screenshot Documentation (Optional)

Take screenshots for documentation:

1. **Desktop Home Page** (1920x1080)
   - Full hero section with logo, title, button
   - Feature cards section
   
2. **Mobile Home Page** (375x667)
   - Vertical layout with stacked elements
   
3. **Button Hover State** (desktop)
   - Capture hover effect (darkened, lifted)
   
4. **Developer Tools - Color Inspection**
   - Show #FF5900 color in computed styles

Save to: `aidlc-docs/construction/build-and-test/screenshots/`

---

## Known Issues / Acceptance Criteria

### Must Pass (Blocking)
- ✅ Logo must be visible and sharp
- ✅ Naranja Anáhuac color (#FF5900) must be present
- ✅ Inter font must load and apply
- ✅ Home page content must match approved text
- ✅ Button must navigate correctly
- ✅ No console errors

### Should Pass (Important)
- ✅ Hover effects should work smoothly
- ✅ Responsive design should work on all devices
- ✅ Border radius should be consistent

### May Vary (Acceptable)
- Font fallback to Segoe UI if Google Fonts blocked (acceptable)
- Slight color variations due to monitor calibration (acceptable if close)
- Minor spacing differences on different browsers (acceptable if functional)

---

## Next Steps

After successful visual verification:
1. **Proceed to functional-test-instructions.md** - Test navigation and features
2. **Proceed to performance-test-instructions.md** - Measure page load time
3. **Document any visual issues** in build-and-test-summary.md

---

**Visual Verification Instructions Complete**  
**Status**: Ready for Functional Testing
