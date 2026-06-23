# ActivityWatch WebUI - Anáhuac Mayab Customization

**Customization Date**: 2026-05-19  
**Institution**: Universidad Anáhuac Mayab  
**Purpose**: Apply institutional branding to ActivityWatch WebUI

## Overview

This is a customized fork of the ActivityWatch WebUI with Anáhuac Mayab institutional branding applied. All functionality remains unchanged - only visual elements have been modified.

## Customizations Applied

### 1. Logo Replacement
- **Original**: ActivityWatch logo
- **New**: Anáhuac Mayab isotipo (black version on transparent background)
- **Location**: `static/logo.png`
- **Format**: PNG with alpha channel (512x512px, 22.5 KB)
- **Usage**: Header (navbar) and Home page

### 2. Color Palette
- **Primary Brand Color**: #FF5900 (Naranja Anáhuac)
- **Complete Palette**: 15 colors defined in `src/style/_globals.scss`
- **Source**: https://forlife.anahuac.mx/colores/
- **Colors**:
  - Primary: #FF5900, #E04F00 (hover), #FFF0E8 (light)
  - Text: #040404 (primary), #262626 (secondary), #9C9C9C (muted)
  - Background: #FFFFFF, #F5F5F5 (alt)
  - Borders: #E0E0E0, #9C9C9C (dark)
  - Semantic: #D32F2F (error), #388E3C (success), #F57C00 (warning), #1976D2 (info)

### 3. Typography
- **Font Family**: Inter (Google Fonts) with fallbacks (Segoe UI, system fonts)
- **Weights**: 400 (Regular), 500 (Medium), 700 (Bold)
- **Applied To**: All text elements (body, headings, buttons)
- **Import**: Google Fonts CDN with preconnect optimization

### 4. Border Radius
- **Small (4px)**: Inputs, badges
- **Medium (8px)**: Buttons, small cards
- **Large (12px)**: Large cards, modals, logo container
- **Consistency**: Applied throughout all components

### 5. Home Page Redesign
- **Content**: Custom Anáhuac Mayab messaging in Spanish
- **Layout**: 
  - Hero section (logo + title + subtitle + description + CTA button)
  - Features section (3 cards: Real-time monitoring, Local privacy, Detailed reports)
  - Resources section (links + university info)
- **Language**: Spanish (Universidad context)
- **CTA Button**: "Comenzar a Monitorear" → `/activity/0/view`

### 6. Button Styles
- **Primary Button**: `.btn-anahuac-primary` (Naranja #FF5900, hover effects, shadow on hover)
- **Outline Button**: `.btn-anahuac-outline` (transparent bg, Naranja border)
- **Interactions**: Smooth transitions, lift effect on hover

## Modified Files

### Assets
- `static/logo.png` - **NEW** (Anáhuac logo, 512x512px PNG)

### Styles
- `src/style/_globals.scss` - **MODIFIED** (color palette, typography variables, border radius)
- `src/style/style.scss` - **MODIFIED** (font families, button styles, container border-radius)

### Components
- `src/views/Home.vue` - **MODIFIED** (complete redesign: template + script + styles)
- `src/components/Header.vue` - **NO CHANGES** (logo path already correct)

### HTML
- `index.html` - **MODIFIED** (Inter font import from Google Fonts)

### Build
- `../Dockerfile.webui` (workspace root) - **MODIFIED** (use local source instead of git clone)

## Backup Files

Original files have been backed up with `.backup` extension:
- `src/style/_globals.scss.backup`
- `src/components/Header.vue.backup`
- `src/views/Home.vue.backup`
- `../Dockerfile.webui.backup`

## Functionality Preservation

✅ **Preserved**:
- All navigation links and routes
- All data visualization components
- All query functionality
- All settings and configuration
- All API integrations
- All watchers and backend communication
- All other views (Activity, Timeline, Stopwatch, etc.)

❌ **NOT Modified**:
- Backend (aw-server, aw-datastore)
- Database (postgresql)
- Queries or data processing
- Any components besides Home.vue
- Navigation, routing, or core functionality

## Build Instructions

### Local Development
```bash
cd aw-webui
npm install
npm run serve
# Open http://localhost:8080
```

### Production Build
```bash
cd aw-webui
npm run build
# Output: dist/ directory
```

### Docker Build
```bash
# From workspace root (aw-server-rust/)
docker compose build aw-webui
docker compose up -d aw-webui
# Access http://localhost:5666
```

## Rollback Instructions

To revert to original ActivityWatch branding:

1. **Restore original Dockerfile**:
   ```bash
   mv Dockerfile.webui.backup Dockerfile.webui
   ```

2. **Remove customized aw-webui**:
   ```bash
   rm -rf aw-webui
   ```

3. **Rebuild Docker image** (will clone original from GitHub):
   ```bash
   docker compose build aw-webui
   docker compose up -d
   ```

## Testing Checklist

### Visual
- [ ] Logo visible in header (desktop + mobile)
- [ ] Logo visible on home page with correct styling
- [ ] Home page displays Anáhuac content
- [ ] Colors match Anáhuac palette (#FF5900 primary)
- [ ] Inter font renders correctly
- [ ] Border radius applied consistently
- [ ] Button hover effects work
- [ ] Feature cards have hover animation

### Functional
- [ ] "Comenzar a Monitorear" button navigates to Activity view
- [ ] All navigation links work (Activity, Timeline, Stopwatch, etc.)
- [ ] Settings link works
- [ ] External links open in new tabs
- [ ] Router links work correctly

### Performance
- [ ] Page load time < 2 seconds
- [ ] No layout shift during load
- [ ] Logo loads without visible delay
- [ ] No console errors or warnings
- [ ] Font loads efficiently (preconnect)

### Compatibility
- [ ] Chrome 90+ renders correctly
- [ ] Firefox 88+ renders correctly
- [ ] Safari 14+ renders correctly
- [ ] Edge 90+ renders correctly

### Responsive
- [ ] Desktop (1920x1080, 1366x768) - proper spacing
- [ ] Tablet (768x1024) - layout adapts
- [ ] Mobile (375x667, 414x896) - logo scales, text readable
- [ ] Hero section padding adjusts on mobile
- [ ] Feature cards stack on mobile

## Technical Details

### Technology Stack
- **Framework**: Vue 2
- **Build Tool**: Vite + vue-cli-service
- **Template Language**: Pug
- **Styling**: SCSS + Bootstrap 4
- **Components**: Bootstrap-Vue
- **Package Manager**: npm

### File Structure
```
aw-webui/
├── index.html               (Font import)
├── package.json
├── src/
│   ├── components/
│   │   └── Header.vue      (Verified - no changes needed)
│   ├── views/
│   │   └── Home.vue        (Complete redesign)
│   └── style/
│       ├── _globals.scss   (Color palette + typography)
│       └── style.scss      (Button styles + font families)
└── static/
    └── logo.png            (Anáhuac logo)
```

### Automation Friendly
**data-testid attributes added** for automated testing:
- `home-cta-button` - "Comenzar a Monitorear" button
- `home-logo` - Home page logo image
- `feature-card-1` - Real-time monitoring card
- `feature-card-2` - Local privacy card
- `feature-card-3` - Detailed reports card

## Support

For questions about this customization, contact the DTI team at Universidad Anáhuac Mayab.

For ActivityWatch issues unrelated to customization, refer to:
- **Original Repository**: https://github.com/ActivityWatch/aw-webui
- **Documentation**: https://activitywatch.readthedocs.org/
- **Forum**: https://forum.activitywatch.net/

## License

This customized version maintains the original MPL-2.0 license from ActivityWatch.

**Original Project**: https://activitywatch.net/  
**License**: Mozilla Public License 2.0

---

**Customization completed**: 2026-05-19  
**AI-DLC Workflow**: Full documentation available in `aidlc-docs/`
