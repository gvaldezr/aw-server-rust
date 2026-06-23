# Especificaciones de Diseño - ActivityWatch WebUI (Anáhuac Mayab)

**Fecha**: 2026-05-19  
**Proyecto**: Personalización visual de aw-webui  
**Branding**: Anáhuac Mayab  
**Referencia**: https://forlife.anahuac.mx/colores/

---

## 1. Resumen Ejecutivo

Este documento define las especificaciones de diseño visual para la personalización del WebUI de ActivityWatch, aplicando la identidad corporativa de Anáhuac Mayab. Los cambios se enfocan en:

- Aplicar la paleta de colores institucional (naranja Anáhuac como color primario)
- Implementar tipografía corporativa (Inter/Segoe UI)
- Establecer border radius consistentes
- Mantener funcionalidad completa sin cambios en queries o componentes backend

---

## 2. Paleta de Colores (Anáhuac Mayab)

Basada en la guía de marca institucional: https://forlife.anahuac.mx/colores/

### Colores Principales

| Variable CSS | Código Hex | Uso |
|--------------|------------|-----|
| `--color-primary` | `#FF5900` | **Naranja Anáhuac** - Botones, acentos, links activos |
| `--color-primary-hover` | `#E04F00` | Hover del primario |
| `--color-primary-light` | `#FFF0E8` | Fondo suave naranja |

### Colores de Texto

| Variable CSS | Código Hex | Uso |
|--------------|------------|-----|
| `--color-text-primary` | `#040404` | Texto principal (casi negro) |
| `--color-text-secondary` | `#262626` | Texto secundario |
| `--color-text-muted` | `#9C9C9C` | Texto deshabilitado/subtítulos |

### Colores de Fondo

| Variable CSS | Código Hex | Uso |
|--------------|------------|-----|
| `--color-background` | `#FFFFFF` | Fondo principal |
| `--color-background-alt` | `#F5F5F5` | Fondo alternativo (páginas internas) |
| `--color-surface` | `#FFFFFF` | Superficies/cards |

### Colores de Bordes

| Variable CSS | Código Hex | Uso |
|--------------|------------|-----|
| `--color-border` | `#E0E0E0` | Bordes suaves |
| `--color-border-dark` | `#9C9C9C` | Bordes con más contraste |

### Colores de Estado

| Variable CSS | Código Hex | Uso |
|--------------|------------|-----|
| `--color-error` | `#D32F2F` | Errores |
| `--color-success` | `#388E3C` | Éxito |
| `--color-warning` | `#F57C00` | Advertencias |
| `--color-info` | `#1976D2` | Información |

### Implementación CSS

```css
:root {
  /* Colores Primarios */
  --color-primary: #FF5900;
  --color-primary-hover: #E04F00;
  --color-primary-light: #FFF0E8;
  
  /* Colores de Texto */
  --color-text-primary: #040404;
  --color-text-secondary: #262626;
  --color-text-muted: #9C9C9C;
  
  /* Colores de Fondo */
  --color-background: #FFFFFF;
  --color-background-alt: #F5F5F5;
  --color-surface: #FFFFFF;
  
  /* Colores de Bordes */
  --color-border: #E0E0E0;
  --color-border-dark: #9C9C9C;
  
  /* Colores de Estado */
  --color-error: #D32F2F;
  --color-success: #388E3C;
  --color-warning: #F57C00;
  --color-info: #1976D2;
}
```

---

## 3. Tipografía

### Font Family

```css
font-family: 'Inter', 'Segoe UI', sans-serif;
```

**Prioridad de Fuentes**:
1. `Inter` - Fuente principal (requiere importar desde Google Fonts o local)
2. `Segoe UI` - Fallback para sistemas Windows
3. `sans-serif` - Fallback genérico

### Importación de Inter (Google Fonts)

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;700&display=swap" rel="stylesheet">
```

### Especificaciones de Tamaño y Peso

| Elemento | Tamaño | Peso | Color | Uso |
|----------|--------|------|-------|-----|
| **Título app (header)** | `1.25rem` (20px) | 700 (Bold) | `--color-primary` | Logo/título en barra superior |
| **Título login** | `1.25rem` (20px) | 700 (Bold) | `--color-text-primary` | Título de página de login |
| **Subtítulo login** | `0.8rem` (12.8px) | 400 (Regular) | `--color-text-muted` | Descripción bajo título login |
| **Texto body** | `0.875rem` (14px) | 400 (Regular) | `--color-text-secondary` | Texto general de contenido |
| **Labels** | `0.875rem` (14px) | 500 (Medium) | `--color-text-primary` | Etiquetas de formularios |
| **Botones** | `0.875rem` (14px) | 500 (Medium) | `#FFFFFF` | Texto en botones (fondo primary) |

### Implementación CSS

```css
/* Font Family Global */
body {
  font-family: 'Inter', 'Segoe UI', sans-serif;
}

/* Título app (header) */
.app-header-title {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--color-primary);
}

/* Título login */
.login-title {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--color-text-primary);
}

/* Subtítulo login */
.login-subtitle {
  font-size: 0.8rem;
  font-weight: 400;
  color: var(--color-text-muted);
}

/* Texto body */
body, p {
  font-size: 0.875rem;
  font-weight: 400;
  color: var(--color-text-secondary);
}

/* Labels */
label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

/* Botones */
button, .btn {
  font-size: 0.875rem;
  font-weight: 500;
  color: #FFFFFF;
  background-color: var(--color-primary);
}

button:hover, .btn:hover {
  background-color: var(--color-primary-hover);
}
```

---

## 4. Border Radius

### Especificaciones

| Variable CSS | Valor | Uso |
|--------------|-------|-----|
| `--border-radius-sm` | `4px` | Inputs, badges, elementos pequeños |
| `--border-radius-md` | `8px` | Botones, cards pequeñas |
| `--border-radius-lg` | `12px` | Cards principales, login container, logo |

### Implementación CSS

```css
:root {
  --border-radius-sm: 4px;
  --border-radius-md: 8px;
  --border-radius-lg: 12px;
}

/* Aplicación por Elemento */
input, textarea, select, .badge {
  border-radius: var(--border-radius-sm);
}

button, .btn, .card-small {
  border-radius: var(--border-radius-md);
}

.card, .login-container, .logo-container {
  border-radius: var(--border-radius-lg);
}
```

---

## 5. Componentes Específicos

### 5.1 Botones

```css
.btn-primary {
  background-color: var(--color-primary);
  color: #FFFFFF;
  font-size: 0.875rem;
  font-weight: 500;
  border: none;
  border-radius: var(--border-radius-md);
  padding: 10px 20px;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.btn-primary:hover {
  background-color: var(--color-primary-hover);
}

.btn-primary:active {
  background-color: #C94500;
}
```

### 5.2 Cards

```css
.card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.card:hover {
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}
```

### 5.3 Inputs

```css
input, textarea, select {
  font-family: 'Inter', 'Segoe UI', sans-serif;
  font-size: 0.875rem;
  color: var(--color-text-primary);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-sm);
  padding: 8px 12px;
  transition: border-color 0.2s ease;
}

input:focus, textarea:focus, select:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-light);
}
```

### 5.4 Links

```css
a {
  color: var(--color-primary);
  text-decoration: none;
  transition: color 0.2s ease;
}

a:hover {
  color: var(--color-primary-hover);
  text-decoration: underline;
}
```

---

## 6. Logo y Branding

### 6.1 Logo de Anáhuac

**Ubicación esperada**: 
- Header/navbar: Logo institucional de Anáhuac Mayab
- Pantalla de inicio: Logo más grande con texto "ActivityWatch - Anáhuac Mayab"

**Especificaciones del Logo**:
- Formato: PNG con transparencia (o SVG)
- Border radius: `var(--border-radius-lg)` (12px)
- Tamaño header: 40px height
- Tamaño inicio: 80-100px height

### 6.2 Container del Logo

```css
.logo-container {
  display: inline-block;
  border-radius: var(--border-radius-lg);
  overflow: hidden;
}

.logo-header {
  height: 40px;
  width: auto;
}

.logo-home {
  height: 80px;
  width: auto;
}
```

---

## 7. Pantalla de Inicio (Landing Page)

### 7.1 Layout Propuesto

```
+------------------------------------------+
|  [Header con Logo Anáhuac]               |
+------------------------------------------+
|                                          |
|  [Logo Grande Centrado]                  |
|  ActivityWatch - Anáhuac Mayab           |
|                                          |
|  Bienvenido al sistema de seguimiento   |
|  de actividades de Anáhuac Mayab        |
|                                          |
|  [Botón: Comenzar]                       |
|                                          |
+------------------------------------------+
```

### 7.2 Estilos de la Pantalla de Inicio

```css
.home-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 80vh;
  background-color: var(--color-background);
  padding: 40px 20px;
}

.home-logo {
  height: 80px;
  width: auto;
  margin-bottom: 24px;
}

.home-title {
  font-size: 1.75rem;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 12px;
  text-align: center;
}

.home-subtitle {
  font-size: 1rem;
  font-weight: 400;
  color: var(--color-text-muted);
  text-align: center;
  max-width: 600px;
  margin-bottom: 32px;
  line-height: 1.6;
}

.home-cta-button {
  background-color: var(--color-primary);
  color: #FFFFFF;
  font-size: 1rem;
  font-weight: 500;
  border: none;
  border-radius: var(--border-radius-md);
  padding: 14px 32px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.home-cta-button:hover {
  background-color: var(--color-primary-hover);
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(255, 89, 0, 0.3);
}
```

---

## 8. Restricciones y Alcance

### 8.1 Componentes a Modificar

✅ **Incluidos en el alcance**:
- Paleta de colores global (CSS variables)
- Tipografía (font-family, tamaños, pesos)
- Logo en header y pantalla de inicio
- Estilos de pantalla de inicio (landing page)
- Botones y componentes UI básicos
- Border radius de todos los elementos

❌ **Excluidos del alcance**:
- Queries de ActivityWatch (mantener sin cambios)
- Componentes de backend (Rust)
- Funcionalidad existente (solo cambios visuales)
- APIs o endpoints
- Lógica de negocio

### 8.2 Compatibilidad

- **Vue.js**: Las clases CSS deben ser compatibles con componentes Vue
- **Responsive**: Los estilos deben funcionar en desktop y móvil
- **Navegadores**: Chrome, Firefox, Safari, Edge (últimas 2 versiones)
- **Dark Mode**: Considerar implementación futura (no en esta fase)

---

## 9. Archivos a Modificar (Estimado)

Basado en la estructura típica de aw-webui (Vue.js):

1. **`src/assets/logo.png`** - Reemplazar con logo de Anáhuac Mayab
2. **`src/style/style.scss`** (o similar) - Variables CSS globales
3. **`src/components/Header.vue`** - Logo en header
4. **`src/views/Home.vue`** (o equivalente) - Pantalla de inicio
5. **`public/index.html`** - Importar fuente Inter desde Google Fonts
6. **`Dockerfile.webui`** - Ajustar para usar fuentes locales si es necesario

---

## 10. Criterios de Éxito

✅ La aplicación debe:
1. Mostrar el logo de Anáhuac Mayab en header y pantalla de inicio
2. Usar la paleta de colores institucional (#FF5900 como primario)
3. Aplicar tipografía Inter en todos los textos
4. Mantener todos los border radius especificados
5. Preservar toda la funcionalidad existente sin errores
6. Pasar validación de build de Docker
7. Ser visualmente consistente con la marca Anáhuac Mayab

---

## 11. Próximos Pasos

1. ✅ Especificaciones de diseño documentadas
2. ⏳ Completar preguntas de requisitos (webui-design-questions.md)
3. ⏳ Obtener archivo de logo de Anáhuac Mayab
4. ⏳ Clonar repositorio aw-webui localmente
5. ⏳ Implementar cambios CSS y componentes
6. ⏳ Actualizar Dockerfile.webui para usar código local
7. ⏳ Build y validación con Docker Compose

---

**Documento creado**: 2026-05-19  
**Última actualización**: 2026-05-19  
**Versión**: 1.0
